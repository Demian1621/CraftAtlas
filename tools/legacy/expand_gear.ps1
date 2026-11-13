# Script to generate comprehensive database with 100+ farms, 25+ dupers, and all enchantable items
$repoRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$dbFile = Join-Path $repoRoot "data\database.json"

# Load existing database
$db = Get-Content -Raw -Encoding utf8 $dbFile | ConvertFrom-Json

# 1. ADD NEW GOD GEAR ITEMS
$newGear = @(
    [PSCustomObject]@{
        id = "fishing_rod"
        type = "tool"
        name_es_mx = "Caña de Pescar de la Suerte"
        name_es_es = "Caña de Pescar de la Suerte"
        name_en = "God Fishing Rod"
        icon = "🎣"
        enchantments = @(
            [PSCustomObject]@{ id = "luck_of_the_sea"; level = "III"; es_mx = "Suerte marina III"; es_es = "Suerte marina III"; en = "Luck of the Sea III"; desc = "Aumenta la probabilidad de pescar tesoros raros (libros con Reparación, arcos y monturas)." },
            [PSCustomObject]@{ id = "lure"; level = "III"; es_mx = "Atracción III"; es_es = "Señuelo III"; en = "Lure III"; desc = "Reduce el tiempo de espera por picada en 15 segundos." },
            [PSCustomObject]@{ id = "unbreaking"; level = "III"; es_mx = "Irrompibilidad III"; es_es = "Irrompibilidad III"; en = "Unbreaking III"; desc = "Multiplica x4 la durabilidad de la caña." },
            [PSCustomObject]@{ id = "mending"; level = "I"; es_mx = "Reparación"; es_es = "Reparación"; en = "Mending"; desc = "Auto-reparación infinita al pescar cualquier pez u objeto con XP." }
        )
        anvil_order = @(
            "1. Juntar Suerte marina III + Atracción III en un libro",
            "2. Juntar Irrompibilidad III + Reparación en un libro",
            "3. Combinar ambos libros y aplicar a la Caña de Pescar."
        )
        incompatibilities = @()
    },
    [PSCustomObject]@{
        id = "shovel"
        type = "tool"
        name_es_mx = "Pala de Netherita"
        name_es_es = "Pala de Netherita"
        name_en = "Netherite Shovel"
        icon = "⛏️"
        enchantments = @(
            [PSCustomObject]@{ id = "efficiency"; level = "V"; es_mx = "Eficiencia V"; es_es = "Eficiencia V"; en = "Efficiency V"; desc = "Excava arena, grava, tierra y nieve de forma instantánea sin esfuerzo." },
            [PSCustomObject]@{ id = "silk_touch"; level = "I"; es_mx = "Toque de seda"; es_es = "Toque de seda"; en = "Silk Touch"; desc = "Recolecta bloques de césped, podzol, micelio y nieve en capas intactos." },
            [PSCustomObject]@{ id = "unbreaking"; level = "III"; es_mx = "Irrompibilidad III"; es_es = "Irrompibilidad III"; en = "Unbreaking III"; desc = "Más de 8,000 bloques de tierra o arena excavados por ciclo de vida." },
            [PSCustomObject]@{ id = "mending"; level = "I"; es_mx = "Reparación"; es_es = "Reparación"; en = "Mending"; desc = "Reparación continua con orbes de experiencia." }
        )
        anvil_order = @(
            "1. Juntar Eficiencia V + Toque de seda (o Fortuna III)",
            "2. Juntar Irrompibilidad III + Reparación",
            "3. Combinar y aplicar a la Pala."
        )
        incompatibilities = @("Fortuna III (excluyente con Toque de seda)")
    },
    [PSCustomObject]@{
        id = "hoe"
        type = "tool"
        name_es_mx = "Azada de Netherita"
        name_es_es = "Azada de Netherita"
        name_en = "Netherite Hoe"
        icon = "🌾"
        enchantments = @(
            [PSCustomObject]@{ id = "efficiency"; level = "V"; es_mx = "Eficiencia V"; es_es = "Eficiencia V"; en = "Efficiency V"; desc = "Destruye bloques de sculk, hojas de árbol, fardos de heno y esponjas al instante." },
            [PSCustomObject]@{ id = "silk_touch"; level = "I"; es_mx = "Toque de seda"; es_es = "Toque de seda"; en = "Silk Touch"; desc = "Recolecta catalizadores y sensores de sculk, bloques de hojas y esponjas intactas." },
            [PSCustomObject]@{ id = "unbreaking"; level = "III"; es_mx = "Irrompibilidad III"; es_es = "Irrompibilidad III"; en = "Unbreaking III"; desc = "Durabilidad multiplicada x4." },
            [PSCustomObject]@{ id = "mending"; level = "I"; es_mx = "Reparación"; es_es = "Reparación"; en = "Mending"; desc = "Reparación perpetua con XP." }
        )
        anvil_order = @(
            "1. Juntar Eficiencia V + Toque de seda",
            "2. Juntar Irrompibilidad III + Reparación",
            "3. Combinar libros y aplicar a la Azada."
        )
        incompatibilities = @("Fortuna III (excluyente con Toque de seda)")
    },
    [PSCustomObject]@{
        id = "shield"
        type = "armor"
        name_es_mx = "Escudo Reforzado"
        name_es_es = "Escudo Reforzado"
        name_en = "Reinforced Shield"
        icon = "🛡️"
        enchantments = @(
            [PSCustomObject]@{ id = "unbreaking"; level = "III"; es_mx = "Irrompibilidad III"; es_es = "Irrompibilidad III"; en = "Unbreaking III"; desc = "Bloquea explosiones de creepers, flechas y golpes pesados sin romperse." },
            [PSCustomObject]@{ id = "mending"; level = "I"; es_mx = "Reparación"; es_es = "Reparación"; en = "Mending"; desc = "Reparación automática con cualquier granja de XP." }
        )
        anvil_order = @("Juntar Irrompibilidad III + Reparación en un libro y aplicarlo al Escudo.")
        incompatibilities = @()
    },
    [PSCustomObject]@{
        id = "shears"
        type = "tool"
        name_es_mx = "Tijeras de Precisión"
        name_es_es = "Cizallas de Precisión"
        name_en = "Precision Shears"
        icon = "✂️"
        enchantments = @(
            [PSCustomObject]@{ id = "efficiency"; level = "V"; es_mx = "Eficiencia V"; es_es = "Eficiencia V"; en = "Efficiency V"; desc = "Corta lana de ovejas, telarañas y enredaderas a velocidad instantánea." },
            [PSCustomObject]@{ id = "unbreaking"; level = "III"; es_mx = "Irrompibilidad III"; es_es = "Irrompibilidad III"; en = "Unbreaking III"; desc = "Durabilidad cuadruplicada." },
            [PSCustomObject]@{ id = "mending"; level = "I"; es_mx = "Reparación"; es_es = "Reparación"; en = "Mending"; desc = "Auto-reparación permanente con experiencia." }
        )
        anvil_order = @("Juntar Eficiencia V + Irrompibilidad III + Reparación y aplicar a las Tijeras.")
        incompatibilities = @()
    },
    [PSCustomObject]@{
        id = "flint_and_steel"
        type = "tool"
        name_es_mx = "Mechero Perpetuo"
        name_es_es = "Pedernal y Acero Perpetuo"
        name_en = "Flint and Steel"
        icon = "🔥"
        enchantments = @(
            [PSCustomObject]@{ id = "unbreaking"; level = "III"; es_mx = "Irrompibilidad III"; es_es = "Irrompibilidad III"; en = "Unbreaking III"; desc = "Enciende cientos de portales del Nether y dinamitas sin desgastarse." },
            [PSCustomObject]@{ id = "mending"; level = "I"; es_mx = "Reparación"; es_es = "Reparación"; en = "Mending"; desc = "Nunca se rompe." }
        )
        anvil_order = @("Juntar Irrompibilidad III + Reparación y aplicar al Mechero.")
        incompatibilities = @()
    },
    [PSCustomObject]@{
        id = "wolf_armor"
        type = "armor"
        name_es_mx = "Armadura de Lobo (1.21+ / 26.3)"
        name_es_es = "Armadura de Lobo (1.21+ / 26.3)"
        name_en = "Wolf Armor"
        icon = "🐺"
        enchantments = @(
            [PSCustomObject]@{ id = "unbreaking"; level = "III"; es_mx = "Irrompibilidad III"; es_es = "Irrompibilidad III"; en = "Unbreaking III"; desc = "Absorbe todo el daño entrante protegiendo la salud de tu lobo domesticado." },
            [PSCustomObject]@{ id = "mending"; level = "I"; es_mx = "Reparación"; es_es = "Reparación"; en = "Mending"; desc = "Se repara absorbiendo orbes de experiencia cuando el lobo mata monstruos." }
        )
        anvil_order = @("Juntar Irrompibilidad III + Reparación y aplicar con el yunque.")
        incompatibilities = @()
    },
    [PSCustomObject]@{
        id = "turtle_helmet"
        type = "armor"
        name_es_mx = "Caparazón de Tortuga"
        name_es_es = "Caparazón de Tortuga"
        name_en = "Turtle Shell Helmet"
        icon = "🐢"
        enchantments = @(
            [PSCustomObject]@{ id = "protection"; level = "IV"; es_mx = "Protección IV"; es_es = "Protección IV"; en = "Protection IV"; desc = "Otorga el efecto de Apnea (+10 segundos extra de respiración al sumergirte en agua)." },
            [PSCustomObject]@{ id = "respiration"; level = "III"; es_mx = "Respiración III"; es_es = "Respiración III"; en = "Respiration III"; desc = "Suma 45 segundos bajo el agua con visión clara." },
            [PSCustomObject]@{ id = "aqua_affinity"; level = "I"; es_mx = "Afinidad acuática"; es_es = "Afinidad acuática"; en = "Aqua Affinity"; desc = "Minado submarino sin penalización de velocidad." },
            [PSCustomObject]@{ id = "unbreaking"; level = "III"; es_mx = "Irrompibilidad III"; es_es = "Irrompibilidad III"; en = "Unbreaking III"; desc = "Durabilidad x4." },
            [PSCustomObject]@{ id = "mending"; level = "I"; es_mx = "Reparación"; es_es = "Reparación"; en = "Mending"; desc = "Reparación continua con XP." }
        )
        anvil_order = @("Combinar Protección IV + Respiración III + Afinidad acuática + Irrompibilidad III + Reparación.")
        incompatibilities = @("Protecciones elementales secundarias")
    }
)

# Append new gear if not present
foreach ($g in $newGear) {
    if (-not ($db.gear | Where-Object { $_.id -eq $g.id })) {
        $db.gear += $g
    }
}

"Updated gear list to $($db.gear.Count) items."
