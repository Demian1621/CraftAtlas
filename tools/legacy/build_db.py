# -*- coding: utf-8 -*-
import json
import os

database = {
    "version": "26.3",
    "last_updated": "Noviembre 2026",
    "supported_loaders": ["Fabric", "NeoForge", "Vanilla"],
    "gear": [
        {
            "id": "helmet",
            "type": "armor",
            "name_es_mx": "Casco de Netherita",
            "name_es_es": "Casco de Netherita",
            "name_en": "Netherite Helmet",
            "icon": "🪖",
            "enchantments": [
                {"id": "protection", "level": "IV", "es_mx": "Protección IV", "es_es": "Protección IV", "en": "Protection IV", "desc": "Reduce 16% todo el daño entrante."},
                {"id": "respiration", "level": "III", "es_mx": "Respiración III", "es_es": "Respiración III", "en": "Respiration III", "desc": "Otorga +45 segundos bajo el agua y visión submarina clara."},
                {"id": "aqua_affinity", "level": "I", "es_mx": "Afinidad acuática", "es_es": "Afinidad acuática", "en": "Aqua Affinity", "desc": "Velocidad de minado submarino al 100% normal."},
                {"id": "unbreaking", "level": "III", "es_mx": "Irrompibilidad III", "es_es": "Irrompibilidad III", "en": "Unbreaking III", "desc": "Multiplica x4 la durabilidad efectiva."},
                {"id": "mending", "level": "I", "es_mx": "Reparación", "es_es": "Reparación", "en": "Mending", "desc": "Auto-reparación permanente con orbes de experiencia."},
                {"id": "thorns", "level": "III", "es_mx": "Espinas III (Opcional)", "es_es": "Espinas III (Opcional)", "en": "Thorns III (Optional)", "desc": "Devuelve daño a atacantes; gasta algo más de durabilidad."}
            ],
            "anvil_order": [
                "1. Casco + Protección IV",
                "2. Combinar Respiración III + Afinidad acuática en un libro",
                "3. Combinar Irrompibilidad III + Reparación en un libro",
                "4. Juntar ambos libros combinados en uno solo",
                "5. Aplicar el libro final al Casco (Evita el cartel 'Demasiado caro')."
            ],
            "incompatibilities": ["Protección contra el fuego", "Protección contra explosiones", "Protección contra proyectiles"]
        },
        {
            "id": "chestplate",
            "type": "armor",
            "name_es_mx": "Pechera de Netherita",
            "name_es_es": "Peto de Netherita",
            "name_en": "Netherite Chestplate",
            "icon": "🦺",
            "enchantments": [
                {"id": "protection", "level": "IV", "es_mx": "Protección IV", "es_es": "Protección IV", "en": "Protection IV", "desc": "Aporta el mayor valor de armadura base del juego con 80% reducción de daño combinada."},
                {"id": "unbreaking", "level": "III", "es_mx": "Irrompibilidad III", "es_es": "Irrompibilidad III", "en": "Unbreaking III", "desc": "Resistencia cuatriplicada."},
                {"id": "mending", "level": "I", "es_mx": "Reparación", "es_es": "Reparación", "en": "Mending", "desc": "Durabilidad infinita con cualquier granja de XP."},
                {"id": "thorns", "level": "III", "es_mx": "Espinas III (Opcional)", "es_es": "Espinas III (Opcional)", "en": "Thorns III (Optional)", "desc": "Refleja daño contra monstruos y jugadores."}
            ],
            "anvil_order": [
                "1. Pechera + Protección IV",
                "2. Juntar Irrompibilidad III + Reparación en un libro",
                "3. Aplicar el libro a la Pechera."
            ],
            "incompatibilities": ["Otras protecciones elementales"]
        },
        {
            "id": "leggings",
            "type": "armor",
            "name_es_mx": "Pantalones de Netherita",
            "name_es_es": "Grebas de Netherita",
            "name_en": "Netherite Leggings",
            "icon": "👖",
            "enchantments": [
                {"id": "protection", "level": "IV", "es_mx": "Protección IV", "es_es": "Protección IV", "en": "Protection IV", "desc": "Defensa general contra todos los tipos de daño."},
                {"id": "swift_sneak", "level": "III", "es_mx": "Sigilo rápido III", "es_es": "Sigilo veloz III", "en": "Swift Sneak III", "desc": "Caminás agachado a velocidad casi normal. Exclusivo de Ciudades del Deep Dark."},
                {"id": "unbreaking", "level": "III", "es_mx": "Irrompibilidad III", "es_es": "Irrompibilidad III", "en": "Unbreaking III", "desc": "Durabilidad quadruplicada."},
                {"id": "mending", "level": "I", "es_mx": "Reparación", "es_es": "Reparación", "en": "Mending", "desc": "Reparación continua con experiencia."}
            ],
            "anvil_order": [
                "1. Pantalones + Protección IV",
                "2. Juntar Sigilo rápido III + Irrompibilidad III",
                "3. Juntar con Reparación y aplicar a los pantalones."
            ],
            "incompatibilities": ["Otras protecciones"]
        },
        {
            "id": "boots",
            "type": "armor",
            "name_es_mx": "Botas de Netherita",
            "name_es_es": "Botas de Netherita",
            "name_en": "Netherite Boots",
            "icon": "👢",
            "enchantments": [
                {"id": "protection", "level": "IV", "es_mx": "Protección IV", "es_es": "Protección IV", "en": "Protection IV", "desc": "Completa el set para alcanzar el tope de 80% reducción de daño."},
                {"id": "feather_falling", "level": "IV", "es_mx": "Caída de pluma IV", "es_es": "Caída de pluma IV", "en": "Feather Falling IV", "desc": "Reduce casi a cero el daño de caídas desde alturas gigantescas."},
                {"id": "depth_strider", "level": "III", "es_mx": "Agilidad acuática III", "es_es": "Agilidad acuática III", "en": "Depth Strider III", "desc": "Caminar y nadar en el agua a velocidad de tierra firme."},
                {"id": "soul_speed", "level": "III", "es_mx": "Velocidad de alma III", "es_es": "Velocidad del alma III", "en": "Soul Speed III", "desc": "Gran velocidad sobre arena y tierra de almas en el Nether."},
                {"id": "unbreaking", "level": "III", "es_mx": "Irrompibilidad III", "es_es": "Irrompibilidad III", "en": "Unbreaking III", "desc": "Durabilidad x4."},
                {"id": "mending", "level": "I", "es_mx": "Reparación", "es_es": "Reparación", "en": "Mending", "desc": "Reparación automática permanente."}
            ],
            "anvil_order": [
                "1. Juntar Protección IV + Caída de pluma IV",
                "2. Juntar Agilidad acuática III + Velocidad de alma III",
                "3. Juntar Irrompibilidad III + Reparación",
                "4. Combinar libros de pasos 1 y 2",
                "5. Combinar con libro de paso 3 y aplicar a las Botas."
            ],
            "incompatibilities": ["Paso helado (Frost Walker) - Excluyente con Agilidad acuática."]
        },
        {
            "id": "elytra",
            "type": "armor",
            "name_es_mx": "Élitros",
            "name_es_es": "Élitros",
            "name_en": "Elytra",
            "icon": "🪽",
            "enchantments": [
                {"id": "unbreaking", "level": "III", "es_mx": "Irrompibilidad III", "es_es": "Irrompibilidad III", "en": "Unbreaking III", "desc": "Vuela más de 28 minutos seguidos sin desgaste excesivo."},
                {"id": "mending", "level": "I", "es_mx": "Reparación", "es_es": "Reparación", "en": "Mending", "desc": "Reparación instantánea absorbiendo orbes de XP."}
            ],
            "anvil_order": ["Juntar Irrompibilidad III + Reparación en un libro y aplicarlo directamente."],
            "incompatibilities": []
        },
        {
            "id": "sword",
            "type": "weapon",
            "name_es_mx": "Espada de Netherita",
            "name_es_es": "Espada de Netherita",
            "name_en": "Netherite Sword",
            "icon": "🗡️",
            "enchantments": [
                {"id": "sharpness", "level": "V", "es_mx": "Filo V", "es_es": "Filo V", "en": "Sharpness V", "desc": "+3 de daño adicional por golpe contra cualquier enemigo."},
                {"id": "looting", "level": "III", "es_mx": "Saqueo III", "es_es": "Botín III", "en": "Looting III", "desc": "Multiplica drops de mobs; cabezas de Wither suben a 5.5% de probabilidad."},
                {"id": "sweeping_edge", "level": "III", "es_mx": "Barrido III", "es_es": "Barrido III", "en": "Sweeping Edge III", "desc": "Daño masivo en área (80% a todos los enemigos circundantes)."},
                {"id": "fire_aspect", "level": "II", "es_mx": "Aspecto de fuego II", "es_es": "Aspecto ígneo II", "en": "Fire Aspect II", "desc": "Quema al objetivo 4 segundos; la carne de animales se cocina al instante."},
                {"id": "unbreaking", "level": "III", "es_mx": "Irrompibilidad III", "es_es": "Irrompibilidad III", "en": "Unbreaking III", "desc": "Durabilidad cuadruplicada."},
                {"id": "mending", "level": "I", "es_mx": "Reparación", "es_es": "Reparación", "en": "Mending", "desc": "Nunca más se rompe."}
            ],
            "anvil_order": [
                "1. Juntar Filo V + Saqueo III",
                "2. Juntar Barrido III + Aspecto de fuego II",
                "3. Juntar Irrompibilidad III + Reparación",
                "4. Combinar libros de pasos 1 y 2",
                "5. Combinar con paso 3 y aplicar a la Espada."
            ],
            "incompatibilities": ["Castigo (Smite)", "Perdición de artrópodos", "Empuje II (opcional omitir para que no vuelen lejos)"]
        },
        {
            "id": "pickaxe_fortune",
            "type": "tool",
            "name_es_mx": "Pico de Fortuna (Netherita)",
            "name_es_es": "Pico de Fortuna (Netherita)",
            "name_en": "Fortune Pickaxe",
            "icon": "⛏️",
            "enchantments": [
                {"id": "efficiency", "level": "V", "es_mx": "Eficiencia V", "es_es": "Eficiencia V", "en": "Efficiency V", "desc": "Velocidad de minado instantánea con baliza de Prisa II."},
                {"id": "fortune", "level": "III", "es_mx": "Fortuna III", "es_es": "Fortuna III", "en": "Fortune III", "desc": "Multiplica hasta x4 diamantes, carbón, hierro crudo, cobre, oro, lapis y redstone."},
                {"id": "unbreaking", "level": "III", "es_mx": "Irrompibilidad III", "es_es": "Irrompibilidad III", "en": "Unbreaking III", "desc": "Más de 8,000 bloques picados por barra de vida."},
                {"id": "mending", "level": "I", "es_mx": "Reparación", "es_es": "Reparación", "en": "Mending", "desc": "Se repara al picar minerales que suelten orbes (carbón, redstone, etc.)."}
            ],
            "anvil_order": [
                "1. Juntar Eficiencia V + Fortuna III",
                "2. Juntar Irrompibilidad III + Reparación",
                "3. Combinar ambos libros y aplicar al Pico."
            ],
            "incompatibilities": ["Toque de seda (Silk Touch)"]
        },
        {
            "id": "pickaxe_silk",
            "type": "tool",
            "name_es_mx": "Pico de Toque de Seda (Netherita)",
            "name_es_es": "Pico de Toque de Seda (Netherita)",
            "name_en": "Silk Touch Pickaxe",
            "icon": "💎",
            "enchantments": [
                {"id": "efficiency", "level": "V", "es_mx": "Eficiencia V", "es_es": "Eficiencia V", "en": "Efficiency V", "desc": "Velocidad de minado máxima."},
                {"id": "silk_touch", "level": "I", "es_mx": "Toque de seda", "es_es": "Toque de seda", "en": "Silk Touch", "desc": "Recolecta cofres de ender intactos, hielo, vidrio, sculk y bloques de menas puras."},
                {"id": "unbreaking", "level": "III", "es_mx": "Irrompibilidad III", "es_es": "Irrompibilidad III", "en": "Unbreaking III", "desc": "Durabilidad quadruplicada."},
                {"id": "mending", "level": "I", "es_mx": "Reparación", "es_es": "Reparación", "en": "Mending", "desc": "Reparación continua con XP."}
            ],
            "anvil_order": [
                "1. Juntar Eficiencia V + Toque de seda",
                "2. Juntar Irrompibilidad III + Reparación",
                "3. Combinar ambos libros y aplicar al Pico."
            ],
            "incompatibilities": ["Fortuna III"]
        },
        {
            "id": "axe",
            "type": "tool",
            "name_es_mx": "Hacha de Netherita (Combate / Tala)",
            "name_es_es": "Hacha de Netherita (Combate / Tala)",
            "name_en": "Netherite Axe",
            "icon": "🪓",
            "enchantments": [
                {"id": "sharpness", "level": "V", "es_mx": "Filo V", "es_es": "Filo V", "en": "Sharpness V", "desc": "Daño masivo por golpe individual (rompe el bloqueo del escudo del rival 5 segundos)."},
                {"id": "efficiency", "level": "V", "es_mx": "Eficiencia V", "es_es": "Eficiencia V", "en": "Efficiency V", "desc": "Tala cualquier tronco en décimas de segundo."},
                {"id": "silk_touch", "level": "I", "es_mx": "Toque de seda", "es_es": "Toque de seda", "en": "Silk Touch", "desc": "Ideal para llevarte melones enteros, colmenas de abejas llenas y estanterías sin romperlas."},
                {"id": "unbreaking", "level": "III", "es_mx": "Irrompibilidad III", "es_es": "Irrompibilidad III", "en": "Unbreaking III", "desc": "Durabilidad extendida."},
                {"id": "mending", "level": "I", "es_mx": "Reparación", "es_es": "Reparación", "en": "Mending", "desc": "Reparación continua con experiencia."}
            ],
            "anvil_order": ["Juntar Filo V + Eficiencia V, luego Toque de seda + Irrompibilidad III + Reparación y aplicar."],
            "incompatibilities": ["Castigo (Smite)", "Fortuna III (si tiene Toque de seda)"]
        },
        {
            "id": "bow",
            "type": "weapon",
            "name_es_mx": "Arco de Infinidad",
            "name_es_es": "Arco de Infinidad",
            "name_en": "Infinity Bow",
            "icon": "🏹",
            "enchantments": [
                {"id": "power", "level": "V", "es_mx": "Poder V", "es_es": "Poder V", "en": "Power V", "desc": "+150% de daño: elimina casi cualquier monstruo común de 1 tiro cargado."},
                {"id": "flame", "level": "I", "es_mx": "Fuego", "es_es": "Fuego", "en": "Flame", "desc": "Enciende las flechas con fuego; detona dinamita a distancia."},
                {"id": "punch", "level": "II", "es_mx": "Retroceso II", "es_es": "Retroceso II", "en": "Punch II", "desc": "Empuja enemigos hacia atrás."},
                {"id": "unbreaking", "level": "III", "es_mx": "Irrompibilidad III", "es_es": "Irrompibilidad III", "en": "Unbreaking III", "desc": "Miles de disparos por arco."},
                {"id": "infinity", "level": "I", "es_mx": "Infinidad", "es_es": "Infinidad", "en": "Infinity", "desc": "Dispara flechas infinitas llevando solo 1 flecha normal en el inventario."}
            ],
            "anvil_order": ["Juntar Poder V + Fuego, luego Retroceso II + Irrompibilidad III + Infinidad y aplicar."],
            "incompatibilities": ["Reparación (Mending) - Excluyente con Infinidad."]
        },
        {
            "id": "crossbow",
            "type": "weapon",
            "name_es_mx": "Ballesta Rápida",
            "name_es_es": "Ballesta Rápida",
            "name_en": "Rapid Crossbow",
            "icon": "🎯",
            "enchantments": [
                {"id": "quick_charge", "level": "III", "es_mx": "Carga rápida III", "es_es": "Carga rápida III", "en": "Quick Charge III", "desc": "Tiempo de carga reducido a solo 0.5 segundos."},
                {"id": "multishot", "level": "I", "es_mx": "Multidisparo", "es_es": "Multidisparo", "en": "Multidisparo", "desc": "Dispara 3 proyectiles (flechas o cohetes de fuegos artificiales) por el costo de 1."},
                {"id": "unbreaking", "level": "III", "es_mx": "Irrompibilidad III", "es_es": "Irrompibilidad III", "en": "Unbreaking III", "desc": "Durabilidad quadruplicada."},
                {"id": "mending", "level": "I", "es_mx": "Reparación", "es_es": "Reparación", "en": "Mending", "desc": "Reparación con experiencia."}
            ],
            "anvil_order": ["Juntar Carga rápida III + Multidisparo + Irrompibilidad III + Reparación."],
            "incompatibilities": ["Perforación IV (Piercing) - Excluyente con Multidisparo."]
        },
        {
            "id": "trident",
            "type": "weapon",
            "name_es_mx": "Tridente de Propulsión (Vuelo en Lluvia)",
            "name_es_es": "Tridente de Propulsión (Vuelo en Lluvia)",
            "name_en": "Riptide Trident",
            "icon": "🔱",
            "enchantments": [
                {"id": "riptide", "level": "III", "es_mx": "Propulsión acuática III", "es_es": "Propulsión acuática III", "en": "Riptide III", "desc": "Te lanza disparado por los aires cuando llueve o estás en el agua. Con élitros es el método de viaje más rápido de Minecraft."},
                {"id": "impaling", "level": "V", "es_mx": "Empalamiento V", "es_es": "Empalamiento V", "en": "Impaling V", "desc": "+12.5 de daño contra criaturas acuáticas."},
                {"id": "unbreaking", "level": "III", "es_mx": "Irrompibilidad III", "es_es": "Irrompibilidad III", "en": "Unbreaking III", "desc": "Durabilidad x4."},
                {"id": "mending", "level": "I", "es_mx": "Reparación", "es_es": "Reparación", "en": "Mending", "desc": "Reparación automática."}
            ],
            "anvil_order": ["Juntar Propulsión acuática III + Empalamiento V + Irrompibilidad III + Reparación."],
            "incompatibilities": ["Lealtad (Loyalty)", "Conductividad (Channeling)"]
        },
        {
            "id": "mace",
            "type": "weapon",
            "name_es_mx": "Maza Pesada (1.21+ / 26.3)",
            "name_es_es": "Maza Pesada (1.21+ / 26.3)",
            "name_en": "Heavy Mace",
            "icon": "🔨",
            "enchantments": [
                {"id": "density", "level": "V", "es_mx": "Densidad V", "es_es": "Densidad V", "en": "Density V", "desc": "+2.5 de daño acumulativo adicional por cada bloque de altura desde el que caigas."},
                {"id": "wind_burst", "level": "III", "es_mx": "Aeroimpulso III", "es_es": "Aeroimpulso III", "en": "Wind Burst III", "desc": "Te dispara hacia arriba tras asestar un golpe en caída para encadenar saltos infinitos."},
                {"id": "fire_aspect", "level": "II", "es_mx": "Aspecto de fuego II", "es_es": "Aspecto ígneo II", "en": "Fire Aspect II", "desc": "Añade quemadura y daño ígneo."},
                {"id": "unbreaking", "level": "III", "es_mx": "Irrompibilidad III", "es_es": "Irrompibilidad III", "en": "Unbreaking III", "desc": "Durabilidad aumentada."},
                {"id": "mending", "level": "I", "es_mx": "Reparación", "es_es": "Reparación", "en": "Mending", "desc": "Reparación con experiencia."}
            ],
            "anvil_order": ["Juntar Densidad V + Aeroimpulso III + Aspecto de fuego II + Irrompibilidad III + Reparación."],
            "incompatibilities": ["Fisura (Breach) es incompatible con Densidad."]
        }
    ],
    "farms": [
        {
            "id": "iron_basic",
            "category": "Hierro",
            "tier": "Fácil",
            "title_es_mx": "Granja de Hierro Compacta (3 Aldeanos)",
            "title_en": "Compact Iron Farm (3 Villagers)",
            "rate": "300 - 350 lingotes / hora",
            "desc": "La granja indispensable para empezar cualquier mundo técnico. Utiliza la mecánica de pánico de 3 aldeanos mirando a un zombi encerrado para invocar golems de hierro en una plataforma de 5x5 hacia una fogata o lava con tolvas.",
            "schematic": "01_Hierro/1_Facil_Hierro_Compacta.litematic",
            "tips": "Los aldeanos deben dormir al menos un instante cada noche para reiniciar el pánico. El zombi debe tener una etiqueta (Name Tag) para no despawnear."
        },
        {
            "id": "iron_medium",
            "category": "Hierro",
            "tier": "Intermedia",
            "title_es_mx": "Granja de Hierro Treehouse Estética",
            "title_en": "Treehouse Aesthetic Iron Farm",
            "rate": "700 - 800 lingotes / hora",
            "desc": "Doble cápsula de aldeanos perfectamente sincronizada y camuflada como un árbol gigante para no romper la estética de tu base.",
            "schematic": "01_Hierro/2_Intermedia_Hierro_Treehouse.litematic",
            "tips": "Línea de visión oscilante con trampilla para regular el pánico de los aldeanos de forma intermitente, maximizando la frecuencia de spawn."
        },
        {
            "id": "iron_hard",
            "category": "Hierro",
            "tier": "Difícil",
            "title_es_mx": "Granja de Hierro Gigante Multi-Célula",
            "title_en": "Giant Multi-Cell Iron Farm",
            "rate": "3,000+ lingotes / hora",
            "desc": "Complejo masivo de 8 a 16 micro-aldeas separadas a 16 bloques de distancia en 3 dimensiones. Todos los golems caen a un canal central con caída y muerte instantánea.",
            "schematic": "01_Hierro/3_Dificil_Hierro_Gigante.litematic",
            "tips": "Requiere respetar estrictamente las cajas delimitadoras de las aldeas para que las camas no se fusionen en un único centro."
        },
        {
            "id": "gold_basic",
            "category": "Oro y XP",
            "tier": "Fácil",
            "title_es_mx": "Plataforma de Oro en Techo del Nether",
            "title_en": "Nether Roof Gold Platform",
            "rate": "Nivel 0 a 30 en 2 min + 800 pepitas/h",
            "desc": "Construida sobre el techo de bedrock a Y=128 en el bioma Nether Wastes. Al estar por encima del techo, ningún otro mob del Nether puede spawnear, concentrando el 100% del mob cap en tu plataforma.",
            "schematic": "02_Oro_y_XP/1_Facil_Oro_Techo_Nether.litematic",
            "tips": "Disparar una flecha a un Zombified Piglin activa la hostilidad en cadena de todos los demás a 40 bloques a la redonda."
        },
        {
            "id": "gold_medium",
            "category": "Oro y XP",
            "tier": "Intermedia",
            "title_es_mx": "Granja de Oro Donut Circular",
            "title_en": "Nether Donut Gold & XP Farm",
            "rate": "Nivel 0 a 30 en 40 segundos + 2,500 pepitas/h",
            "desc": "Plataformas circulares concéntricas de bloques de magma con un huevo de tortuga en el centro rodeado de trampillas. Los piglins caen por un agujero de 23 bloques y mueren de 1 golpe.",
            "schematic": "02_Oro_y_XP/2_Intermedia_Oro_Donut_Nether.litematic",
            "tips": "Los bloques de magma permiten el spawn exclusivo de Piglins zombificados e impiden la aparición de Ghasts y Endermans."
        },
        {
            "id": "gold_hard",
            "category": "Oro y XP",
            "tier": "Difícil",
            "title_es_mx": "Granja de Oro y Barter Masivo con Piglins",
            "title_en": "Gold Farm & Piglin Bartering Hub",
            "rate": "10,000+ pepitas/h + 50 cofres de Barter/h",
            "desc": "El oro obtenido se craftea automáticamente con bloques Crafter (1.21) y se inyecta por tolvas a una cámara con 24 Piglins en tradeo continuo.",
            "schematic": "02_Oro_y_XP/3_Dificil_Oro_y_Barter_Piglins.litematic",
            "tips": "Genera cuarzo infinito, perlas de ender, libros con Velocidad de alma, pociones y flechas espectrales sin interacción humana."
        },
        {
            "id": "creeper_basic",
            "category": "Pólvora y Creepers",
            "tier": "Fácil",
            "title_es_mx": "Granja de Creepers con Muñeco de Nieve",
            "title_en": "Snow Golem Creeper Farm",
            "rate": "250 pólvora / hora",
            "desc": "Cápsula de 2 pisos con trampillas en el techo (altura 1.8 bloques) para evitar zombis y esqueletos, y alfombras cada 2 bloques para evitar arañas. Solo spawnean creepers.",
            "schematic": "03_Polvora_y_Mobs/1_Facil_Creeper_Snow_Golem.litematic",
            "tips": "Un muñeco de nieve detrás de una valla llama la atención de los creepers, quienes intentan atacarlo y caen por trampillas abiertas."
        },
        {
            "id": "creeper_medium",
            "category": "Pólvora y Creepers",
            "tier": "Intermedia",
            "title_es_mx": "Torre de Creepers Multinivel con Gatos",
            "title_en": "Multi-Floor Creeper Tower with Cats",
            "rate": "1,200 - 1,500 pólvora / hora",
            "desc": "Torre hexagonal de 6 a 8 pisos con gatos domesticados en el centro de cada piso. Los creepers huyen aterrorizados de los gatos y se arrojan a corrientes de agua.",
            "schematic": "03_Polvora_y_Mobs/2_Intermedia_Creeper_Tower.litematic",
            "tips": "Al vaciar los pisos en menos de 2 segundos, el mob cap se renueva constantemente generando un flujo ininterrumpido de pólvora para cohetes de élitros."
        },
        {
            "id": "creeper_hard",
            "category": "Pólvora y Creepers",
            "tier": "Difícil",
            "title_es_mx": "Granja de Brujas de Pantano (Witch Farm)",
            "title_en": "Swamp Witch Farm Perimetral",
            "rate": "Pólvora, Redstone, Glowstone y Palos masivos",
            "desc": "Construida exactamente dentro de la caja delimitadora (7x9x7) de una cabaña de bruja en un pantano. Requiere spawn-proof de las cuevas circundantes o un perímetro.",
            "schematic": "03_Polvora_y_Mobs/3_Dificil_Witch_Farm_Pantano.litematic",
            "tips": "Pistones con relojes de redstone empujan a las brujas a una cámara de trituración o caída para recolectar todos los drops de pociones del juego."
        },
        {
            "id": "raid_hard",
            "category": "Endgame y Tótems",
            "tier": "Difícil",
            "title_es_mx": "Stacking Raid Farm (Ianxofour)",
            "title_en": "Stacking Raid Farm (Ianxofour)",
            "rate": "5,000+ esmeraldas/h + Tótems y Brujas infinitos",
            "desc": "La cúspide del juego técnico. Se construye sobre el océano en una sub-división de sub-chunks específica. Permite encadenar oleadas de asaltos continuos en un bucle infinito.",
            "schematic": "06_Endgame_y_Especiales/3_Dificil_Stacking_Raid_Farm_Ianxofour.litematic",
            "tips": "El jugador ataca capitanes mientras el aldeano superior reinicia la raid al instante. Requiere NO ROTAR el esquema bajo ninguna circunstancia."
        }
    ],
    "dupes": [
        {
            "id": "tnt_duper",
            "title": "Duplicador de Dinamita (TNT Duper)",
            "status": "100% Funcional en 26.3 Java",
            "difficulty": "Fácil",
            "components": ["1 Pistón Pegajoso", "6 Bloques de Slime", "1 Ventilador de Coral de Tubo Muerto", "1 Vía Curvada / Detector", "1 Vagoneta", "1 TNT"],
            "how_it_works": "Aprovecha una desincronización en la actualización de estados de bloques de Minecraft Java. Cuando el pistón se extiende, mueve el bloque de slime que transporta la TNT y el coral muerto. El motor de físicas actualiza la TNT como entidad encendida antes de eliminar el bloque sólido original, generando una copia activada que cae mientras el bloque de TNT original permanece intacto en el mecanismo.",
            "use_cases": ["Excavadores de túneles automáticos (World Eaters)", "Cámaras de explosión para granjas de árboles masivas", "Perímetros limpios hasta Bedrock"]
        },
        {
            "id": "gravity_duper",
            "title": "Duplicador de Bloques de Gravedad (Arena, Grava, Concreto)",
            "status": "100% Funcional en 26.3 Java",
            "difficulty": "Intermedia",
            "components": ["1 Portal del End activo", "2 Pistones Normales", "Repetidores de Redstone (reloj de 4 ticks)", "Vallas"],
            "how_it_works": "Al empujar bloques de arena o polvo de concreto con un pistón en el tick exacto en que caen a través del marco horizontal del portal del End, el juego convierte el bloque en una 'FallingBlock' entidad al mismo tiempo que lo teleporta al End. Al llegar a la plataforma del End se materializa el bloque nuevo mientras el original se regenera en el Overworld.",
            "use_cases": ["Hormigón / Concreto infinito para construcciones masivas", "Arena infinita para vidrio y dinamita"]
        },
        {
            "id": "string_duper",
            "title": "Duplicador de Hilos (Tripwire String Duper)",
            "status": "100% Funcional en 26.3 Java",
            "difficulty": "Muy Fácil (Primeros 10 minutos)",
            "components": ["2 Ganchos de Cuerda (Tripwire Hooks)", "1 Hilo", "2 Trampillas", "1 Cubo de Agua"],
            "how_it_works": "Al colocar dos ganchos enfrentados con un hilo y colocar agua corriendo por debajo que toque la base del gancho al abrir una trampilla, el hilo se rompe soltando el ítem mientras el estado del gancho cree que el cable sigue conectado, reponiéndolo instantáneamente.",
            "use_cases": ["Esmeraldas infinitas tradeando con aldeanos Flecheros (Fletcher: 14 hilos = 1 esmeralda)", "Lana infinita crafteando 4 hilos"]
        },
        {
            "id": "bedrock_breaker",
            "title": "Rompimiento de Bedrock (1-Tick Piston)",
            "status": "100% Funcional en 26.3 Java",
            "difficulty": "Intermedia",
            "components": ["1 Pistón Normal", "1 TNT", "1 Palanca / Trampilla", "1 Bloque de Obsidiana"],
            "how_it_works": "Cuando un pistón extendido se destruye en el mismo tick en que explota una dinamita y se coloca un nuevo pistón mirando en sentido opuesto en el espacio del pistón anterior (headless piston), el juego elimina el bloque de Bedrock contiguo para reconstruir la base del pistón.",
            "use_cases": ["Romper el techo del Nether a Y=128 para instalar granjas de oro", "Crear entradas y salidas seguras al vacío del End"]
        },
        {
            "id": "chunk_loader",
            "title": "Cargador de Chunks Permanente (Nether Portal Loop)",
            "status": "100% Funcional en 26.3 Java",
            "difficulty": "Fácil",
            "components": ["1 Portal del Nether", "2 Soltadores (Droppers)", "1 Ítem cualquiera", "Placas de presión / Redstone"],
            "how_it_works": "Cada vez que una entidad atraviesa un portal del Nether, el juego carga los 3x3 chunks que rodean al portal durante 15 segundos (300 ticks de juego). Si construyes un bucle donde un soltador arroja un ítem al portal y del otro lado una tolva lo devuelve, el chunk permanece activo al 100% sin importar en qué dimensión o lugar del mundo te encuentres.",
            "use_cases": ["Mantener granjas de hierro y hornos funcionando mientras exploras a miles de bloques"]
        }
    ],
    "pro_tips": [
        {
            "id": "tip_spawning_sphere",
            "category": "Spawning y Mob Cap",
            "title": "La Esfera de Spawn (24 a 128 bloques)",
            "text": "Los mobs hostiles NUNCA pueden spawnear a menos de 24 bloques del jugador (en distancia euclidiana esférica). A más de 128 bloques, DESPAWNEAN EN 1 TICK exacto. Para maximizar cualquier granja, tu punto AFK debe estar exactamente a 120-125 bloques por encima de las zonas donde no quieres que aparezcan mobs."
        },
        {
            "id": "tip_subchunk_height",
            "category": "Spawning y Mob Cap",
            "title": "La Regla de la Altura Y (Sub-chunks)",
            "text": "En Minecraft Java, el algoritmo de spawning busca bloques disponibles desde Y=-64 hacia arriba. Cuanto más baja sea la coordenada Y de tu granja (por ejemplo, a nivel Y=-58 en el fondo del mundo), MÁS RÁPIDA es la tasa de generación de mobs por segundo. Una granja en el fondo del mundo produce hasta 3 veces más que la misma granja a Y=200."
        },
        {
            "id": "tip_light_level_zero",
            "category": "Spawning y Mob Cap",
            "title": "Nivel de Luz 0 Absoluto",
            "text": "Desde las versiones modernas, los mobs comunes (esqueletos, zombis, creepers) y los Wither Skeletons requieren estrictamente nivel de luz de bloque igual a CERO (0). Cualquier antorcha o bloque de lava con nivel 1 anula por completo la aparición de monstruos en ese bloque."
        },
        {
            "id": "tip_villager_trading_hours",
            "category": "Aldeanos y Economía",
            "title": "Horarios de Trabajo de los Aldeanos",
            "text": "Los aldeanos solo pueden reponer sus intercambios durante su jornada laboral: entre los ticks de día 2000 y 9000 (aproximadamente desde las 8:00 AM hasta las 3:00 PM dentro del juego). No reponen stock de noche ni durante la lluvia si no pueden acceder a su mesa de trabajo."
        },
        {
            "id": "tip_zombie_curing",
            "category": "Aldeanos y Economía",
            "title": "Regla de Curación Zombi (Dificultad Difícil)",
            "text": "Siempre juega en dificultad Difícil (Hard) para curar aldeanos. En Difícil, la conversión a aldeano zombi es 100% garantizada. En Normal hay 50% de probabilidad de que el aldeano muera y se pierda para siempre. En Fácil nunca se convierten (100% muerte)."
        },
        {
            "id": "tip_nether_coords",
            "category": "Nether y Transporte",
            "title": "Matemática de Coordenadas del Nether (1 a 8)",
            "text": "1 bloque recorrido en el Nether equivale exactamente a 8 bloques en el Overworld en los ejes X y Z. La coordenada Y es 1:1. Para conectar dos portales a la perfección sin desincronizaciones, toma las coordenadas de tu base en el Overworld, divide X y Z entre 8, y construye el portal del Nether exactamente en esas coordenadas calculadas."
        },
        {
            "id": "tip_ice_boat_highway",
            "category": "Nether y Transporte",
            "title": "Autopistas de Hielo Azul con Bote",
            "text": "Conducir un bote sobre Hielo Azul (Blue Ice) en el techo del Nether permite viajar a 72 bloques por segundo en el Nether, lo que equivale a casi 600 bloques por segundo en el Overworld. Es 10 veces más rápido que volar con los élitros más veloces."
        },
        {
            "id": "tip_anvil_pyramid",
            "category": "Encantamientos y Yunque",
            "title": "La Pirámide del Yunque (Evitar '¡Demasiado Caro!')",
            "text": "El yunque cobra niveles exponenciales según cuántas veces un ítem pasó por él (penalización acumulada). Si metes libros de uno en uno a la espada, al quinto libro te saldrá el cartel rojo de '¡Demasiado caro!'. La solución es combinar en pares de árbol binario: Libro A + Libro B = Libro AB; Libro C + Libro D = Libro CD; luego Libro AB + Libro CD; y finalmente aplicarlo al arma."
        },
        {
            "id": "tip_mending_offhand",
            "category": "Encantamientos y XP",
            "title": "Prioridad de Reparación con Mano Secundaria",
            "text": "Cuando recoges orbes de experiencia, el juego elige al azar una pieza equipada con Reparación que esté dañada. Si quieres reparar un pico o espada ultra rápido, sostenlo en tu mano secundaria (offhand con tecla F) mientras matas mobs o recoges carbón; toda la XP se canalizará a esa herramienta."
        },
        {
            "id": "tip_elytra_angle",
            "category": "Supervivencia y Combate",
            "title": "Ángulo Óptimo de Vuelo con Élitros",
            "text": "Al activar un cohete de fuegos artificiales con élitros, mantén el cabeceo entre 35° y 40° hacia arriba durante el impulso inicial y luego baja suavemente a 0° para planear. Usar este ángulo duplica la distancia recorrida por cada cohete en comparación con mirar recto hacia adelante."
        },
        {
            "id": "tip_hopper_locking",
            "category": "Redstone y Optimización",
            "title": "Bloqueo de Tolvas para Cero Lag (Server Friendly)",
            "text": "Una tolva desbloqueada chequea el inventario de bloques cada tick de juego. En granjas gigantes con cientos de tolvas, esto destruye los TPS del servidor. Colocar un compostador lleno con trampilla encima de cada tolva, o bloquearlas con señal de redstone cuando no se usan, elimina el 90% del impacto de rendimiento."
        },
        {
            "id": "tip_target_block_redstone",
            "category": "Redstone y Optimización",
            "title": "Bloque de Diana como Redirigidor de Señal",
            "text": "El Bloque de Diana (Target Block) atrae automáticamente el polvo de redstone adyacente hacia sí mismo sin necesidad de repetidores, permitiendo construir circuitos compactos a 1 bloque de ancho sin que los cables vecinos se fusionen entre sí."
        },
        {
            "id": "tip_mining_levels",
            "category": "Minería y Recursos",
            "title": "Alturas Óptimas de Minería en 1.21+ / 26.3",
            "text": "Diamantes: Y=-58 (justo por encima de la capa de Bedrock). Netherita / Ancient Debris: Y=15 en el Nether (usar camas o dinamita). Hierro: Y=232 en cumbres montañosas o Y=16 en llanuras. Oro: Y=-16 en todo el mundo o cualquier altura en bioma Badlands/Mesa."
        },
        {
            "id": "tip_ominous_bottle",
            "category": "Mecánicas 1.21+ / 26.3",
            "title": "Frasco de Presagio y Cámaras de Juicio (Trial Chambers)",
            "text": "En las versiones modernas, matar un capitán de asalto ya no te da el efecto de Mal Presagio automáticamente; en su lugar, suelta un Frasco de Presagio (Ominous Bottle). Esto te permite guardar el efecto en cofres para activar granjas de raids exactamente cuando tú decidas tomando el frasco."
        },
        {
            "id": "tip_fortune_crops",
            "category": "Cultivos y Agricultura",
            "title": "Fortuna III Funciona en Cultivos",
            "text": "Cosechar papas, zanahorias, semillas de trigo y verrugas del Nether con una herramienta que tenga Fortuna III multiplica las cantidades obtenidas hasta en un 300%. No funciona con caña de azúcar, melones ni calabazas completas."
        }
    ]
}

out_path = os.path.normpath(
    os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "data", "database.json")
)
os.makedirs(os.path.dirname(out_path), exist_ok=True)
with open(out_path, "w", encoding="utf-8") as f:
    json.dump(database, f, ensure_ascii=False, indent=2)

print("Database written!")
