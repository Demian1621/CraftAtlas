use std::cmp::Ordering;

use super::models::{EnchantmentItem, GearItem, Language};

/// A node in the Anvil merge tree (either Base Item, Enchanted Book, or Combined product).
#[derive(Debug, Clone)]
struct CombineNode {
    label: String,
    ench_cost: u32,
    uses: u32,
    is_base_item: bool,
}

impl CombineNode {
    fn prior_work_penalty(&self) -> u32 {
        if self.uses == 0 {
            0
        } else {
            (1u32 << self.uses) - 1
        }
    }
}

/// One step in the optimal Anvil merge plan.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AnvilMergeStep {
    pub step_num: usize,
    pub target_label: String,
    pub sacrifice_label: String,
    pub result_label: String,
    pub level_cost: u32,
    pub target_pwc: u32,
    pub sacrifice_pwc: u32,
    pub result_uses: u32,
    pub too_expensive: bool,
}

/// Complete comparison between Optimal Binary Tree vs Naive Linear Anvil order.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AnvilOptimizationPlan {
    pub steps: Vec<AnvilMergeStep>,
    pub total_levels: u32,
    pub max_step_cost: u32,
    pub final_uses: u32,
    pub naive_total_levels: u32,
    pub naive_max_step: u32,
    pub naive_hits_too_expensive: bool,
}

/// Official Minecraft Java Edition enchantment anvil cost multiplier per level.
pub fn enchant_book_multiplier(id: &str) -> u32 {
    match id {
        // Common Weight (Multiplier 1 from book in Java)
        "protection" | "feather_falling" | "sharpness" | "efficiency" | "power" | "loyalty"
        | "unbreaking" | "smite" | "bane_of_arthropods" | "fire_protection" | "projectile_protection"
        | "knockback" | "quick_charge" | "piercing" | "density" => 1,

        // Uncommon Weight (Multiplier 2 from book)
        "blast_protection" | "respiration" | "aqua_affinity" | "depth_strider" | "frost_walker"
        | "looting" | "sweeping_edge" | "fire_aspect" | "flame" | "punch" | "multishot"
        | "riptide" | "impaling" | "fortune" | "luck_of_the_sea" | "lure" | "mending"
        | "wind_burst" | "breach" => 2,

        // Rare / Treasure Book Weight (Multiplier 4 from book)
        "thorns" | "soul_speed" | "swift_sneak" | "silk_touch" | "infinity" | "channeling"
        | "vanishing_curse" | "binding_curse" => 4,

        _ => 2,
    }
}

/// Computes individual Enchanted Book Anvil base cost (`multiplier * level`).
pub fn book_cost(enchant_id: &str, level_num: u8) -> u32 {
    let lv = (level_num.max(1)) as u32;
    enchant_book_multiplier(enchant_id) * lv
}

/// Computes an optimal Binary-Tree Anvil Order avoiding the 40-level "Too Expensive!" cap.
pub fn compute_optimal_anvil_plan(item: &GearItem, lang: Language) -> AnvilOptimizationPlan {
    let base_name = item.name.get(lang).to_string();
    let mut nodes: Vec<CombineNode> = Vec::new();

    nodes.push(CombineNode {
        label: base_name.clone(),
        ench_cost: 0,
        uses: 0,
        is_base_item: true,
    });

    for e in &item.enchantments {
        // Skip base armor stats that aren't anvil-applied books
        if is_passive_stat_entry(e) {
            continue;
        }
        let lvl_str = if e.level.is_empty() {
            String::new()
        } else {
            format!(" {}", e.level)
        };
        let cost = book_cost(&e.id, e.level_num);
        nodes.push(CombineNode {
            label: format!("{}{}", e.name.get(lang), lvl_str),
            ench_cost: cost,
            uses: 0,
            is_base_item: false,
        });
    }

    if nodes.len() <= 1 {
        return AnvilOptimizationPlan {
            steps: Vec::new(),
            total_levels: 0,
            max_step_cost: 0,
            final_uses: 0,
            naive_total_levels: 0,
            naive_max_step: 0,
            naive_hits_too_expensive: false,
        };
    }

    // Naive baseline: apply books strictly left-to-right onto the base item
    let (naive_total, naive_max) = simulate_naive(&nodes);

    // Optimal: Huffman-like balanced binary tree pairing lowest-use / balanced weights so
    // base item keeps uses minimal and max single-step XP stays minimal!
    let (steps, final_uses) = solve_optimal_tree(&nodes, &base_name);

    let total_levels = steps.iter().map(|s| s.level_cost).sum();
    let max_step_cost = steps.iter().map(|s| s.level_cost).max().unwrap_or(0);

    AnvilOptimizationPlan {
        steps,
        total_levels,
        max_step_cost,
        final_uses,
        naive_total_levels: naive_total,
        naive_max_step: naive_max,
        naive_hits_too_expensive: naive_max >= 40,
    }
}

fn is_passive_stat_entry(e: &EnchantmentItem) -> bool {
    e.id == "armor_base" || e.id == "turtle_effect" || e.id == "shield_block" || e.id == "lodestone_bind"
}

fn simulate_naive(nodes: &[CombineNode]) -> (u32, u32) {
    let mut target = nodes[0].clone();
    let mut total = 0u32;
    let mut max_step = 0u32;
    for book in &nodes[1..] {
        let step_cost = target.prior_work_penalty() + book.prior_work_penalty() + book.ench_cost;
        total += step_cost;
        max_step = max_step.max(step_cost);
        target.ench_cost += book.ench_cost;
        target.uses += 1;
    }
    (total, max_step)
}

fn solve_optimal_tree(initial_nodes: &[CombineNode], base_name: &str) -> (Vec<AnvilMergeStep>, u32) {
    // Separate base item from books; pair expensive books carefully at equal heights!
    let base = initial_nodes[0].clone();
    let mut books: Vec<CombineNode> = initial_nodes[1..].to_vec();

    // Put highest-cost enchantments slightly deeper on target (cheaper sacrifice!)
    books.sort_by(|a, b| b.ench_cost.cmp(&a.ench_cost));

    let mut steps = Vec::new();
    let mut step_idx = 1;

    // Include base item directly as one leaf alongside books for true binary balancing:
    let mut pool = Vec::with_capacity(books.len() + 1);
    pool.push(base);
    pool.extend(books);

    // Pair nodes level-by-level (keeps tree depth <= ceil(log2(N)), eliminating exponential penalties)
    while pool.len() > 1 {
        // Sort by uses ASC, then total cost DESC
        pool.sort_by(|a, b| {
            match a.uses.cmp(&b.uses) {
                Ordering::Equal => {
                    // Base item prefers smallest penalty pairing; higher ench_cost acts as target
                    b.ench_cost.cmp(&a.ench_cost)
                }
                other => other,
            }
        });

        let mut next_pool = Vec::new();
        let n = pool.len();
        let mut i = 0;
        // If odd count and bottom-most elements have lower uses, leave 1 node to bubble up
        while i + 1 < n {
            // Pair adjacent elements only when they are at compatible depths, or pair symmetrically
            let a = &pool[i];
            let b = &pool[i + 1];

            // Orient merge: Base item MUST always be target (left slot) if involved; otherwise heavier cost stays target
            let (target, sacrifice) = if b.is_base_item || (!a.is_base_item && b.ench_cost > a.ench_cost) {
                (b.clone(), a.clone())
            } else {
                (a.clone(), b.clone())
            };

            let t_pwc = target.prior_work_penalty();
            let s_pwc = sacrifice.prior_work_penalty();
            let step_cost = t_pwc + s_pwc + sacrifice.ench_cost;
            let new_uses = target.uses.max(sacrifice.uses) + 1;
            let combined_is_base = target.is_base_item || sacrifice.is_base_item;
            let result_label = if combined_is_base {
                format!("{} [Step {}]", base_name, step_idx)
            } else {
                format!("Book #{}", step_idx)
            };

            steps.push(AnvilMergeStep {
                step_num: step_idx,
                target_label: target.label.clone(),
                sacrifice_label: sacrifice.label.clone(),
                result_label: result_label.clone(),
                level_cost: step_cost,
                target_pwc: t_pwc,
                sacrifice_pwc: s_pwc,
                result_uses: new_uses,
                too_expensive: step_cost >= 40,
            });
            step_idx += 1;

            next_pool.push(CombineNode {
                label: result_label,
                ench_cost: target.ench_cost + sacrifice.ench_cost,
                uses: new_uses,
                is_base_item: combined_is_base,
            });
            i += 2;
        }
        if i < n {
            next_pool.push(pool[i].clone());
        }
        pool = next_pool;
    }

    let final_uses = pool.first().map(|n| n.uses).unwrap_or(0);
    (steps, final_uses)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::loader::load_database;

    #[test]
    fn god_boots_never_exceeds_39_cap() {
        let db = load_database().expect("db parses");
        let boots = db.gear.iter().find(|g| g.id == "boots").expect("boots present");
        let plan = compute_optimal_anvil_plan(boots, Language::EnUs);
        assert!(!plan.steps.is_empty());
        assert!(
            plan.max_step_cost <= 39,
            "Optimal boots max step {} exceeded Java 39 level cap!",
            plan.max_step_cost
        );
        assert!(plan.naive_max_step > plan.max_step_cost);
    }
}
