# Pipeline dei contenuti e struttura del progetto

Ultimo aggiornamento: 8 novembre 2026
Branch stabile: `main`  
Branch di lavoro per il refactoring: `refactor/src-domain-layout`  
SHA iniziale del branch di refactoring: `454874a358a18e7e565043d797e3c42ddc46323f` (creato dallo stesso SHA di `main`)  
Ultimo commit di codice verificato prima del refactoring: `55e001ec7dc06f5d9bbb3107f3b5ef3f96354b38` (`Update main.rs`)  
Stato: Fase 3 completata; refactoring strutturale obbligatorio da eseguire prima della Fase 4.  
CI di riferimento: Run #522, verde su Linux e Windows, 660 test superati per piattaforma e avvio headless verificato.

---

## 1. Identità e obiettivi del progetto

Action RPG cooperativo in terza persona, incentrato sui dungeon e ispirato a World of Warcraft Classic e Fellowship. Il progetto non intende riprodurre un client o un server di World of Warcraft.

### Riferimenti tecnici

- Repository: https://github.com/konakona185-gif/wow-friendship-crawler
- Linguaggio: Rust.
- Motore: Bevy 0.18.1.
- Fisica: Avian3D 0.6.
- Navigazione: Bevy Landmass.
- Base dei contenuti: World of Warcraft Classic 1.12.1, build 5875.
- Client di riferimento: Benilla.
- Revisione Benilla: `135ce8889217bf268d1dadbbabcba56df80f3108`.
- Unità di misura: 1 unità di gioco corrisponde a 1 yard di World of Warcraft.

### Fondamenti di gioco

I personaggi iniziano al livello 60. Il progetto è focalizzato sui dungeon e sulle attività cooperative: non è prevista una progressione tradizionale dal livello 1 al 60 né una campagna open world basata su missioni. Le abilità devono utilizzare il grado massimo disponibile, salvo casi in cui un grado inferiore abbia una funzione specifica.

Razze di riferimento: Human, Dwarf, Night Elf, Gnome, Orc, Undead, Tauren e Troll.

Classi di riferimento: Warrior, Paladin, Hunter, Rogue, Priest, Shaman, Mage, Warlock e Druid.

Sono approvate combinazioni personalizzate come Dwarf Shaman e Tauren Paladin. La razza determina le classi disponibili; il ruolo non sostituisce la classe. Ruoli Fellowship: Tank, Healer e DPS. Il Druid deve poter ricoprire tutti e tre i ruoli.

Difficoltà previste per dungeon e contenuti raid trattati come dungeon: Normal, Heroic, Mythic, Mythic+ e Ascended. Ascended è una difficoltà personalizzata ispirata ad Ascension. Il gioco deve essere utilizzabile sia in multiplayer cooperativo sia in single-player.

## 2. Principi architetturali e decisioni permanenti

L'architettura deve essere modulare, estendibile e data-driven. I contenuti devono essere separati dal codice che ne implementa il comportamento.

- RON per le definizioni, Serde per serializzazione/deserializzazione e Rust per regole e runtime.
- Classic 1.12.1 è il primo pacchetto di contenuti, non il limite tecnico del motore.
- In futuro dovrà essere possibile aggiungere contenuti originali o ispirati a espansioni successive: abilità, talenti, specializzazioni, classi, effetti, statistiche, formule, risorse e scuole di danno.
- Le meccaniche già supportate dal motore devono essere configurabili tramite dati. Una meccanica nuova richiede un'implementazione specifica, da rendere riutilizzabile.
- Evitare duplicazioni, responsabilità sovrapposte e dipendenze eccessive dalle singole classi.
- Preferire componenti e risorse ECS riutilizzabili; valutare confini modulari chiari in base alle responsabilità a qualunque dimensione dei file ed evitare frammentazioni artificiali.
- Non modificare meccaniche esistenti senza spiegare prima le conseguenze e ottenere l'approvazione dell'utente.

### Decisioni sul combattimento

- `HealthPool` è la fonte autorevole della salute; `CombatStats` conserva valori legacy e non deve diventare una seconda fonte di verità.
- La threat usa modifiche signed e non può scendere sotto zero.
- Gli effetti reattivi accodati durante un Update vengono eseguiti nell'Update successivo.
- Gli eventi `OnExpire` mantengono come sorgente il caster originale e come target il proprietario dell'aura.
- Le aure supportano scope `Shared` e `PerCaster`, stacking configurabile, refresh e Pandemic.
- Pandemic conserva la durata residua entro il limite del 30% della durata base e la aggiunge alla nuova durata.
- Gli effetti periodici supportano scaling per stack, disattivabile oppure lineare.
- Le scuole composite applicano soltanto la mitigazione più alta tra i componenti, mai la somma.
- Per Physical + magia si confrontano le percentuali effettive di mitigazione fisica e magica.
- Interrupt runtime completo, cast e channel sono assegnati alla Fase 4.

### Fazioni runtime e Summon

La fazione runtime di un'evocazione deriva dalla squadra del proprietario, non da Alliance/Horde:

- `Character` giocante → `Allies`.
- `Enemy` ostile → `Enemies`.
- Evocazione alleata che evoca → evocazione alleata.
- Evocazione nemica che evoca → evocazione nemica.

Le evocazioni alleate sono NPC controllati dall'IA, non `Character` giocanti. Le evocazioni nemiche sono entità ostili. Non assegnare `Enemy` a un'evocazione alleata per riutilizzare impropriamente i sistemi esistenti.

Il comportamento previsto comprende spawn, statistiche, durata, despawn, morte, cleanup dell'owner, navigazione, acquisizione dei bersagli, threat e combattimento.

## 3. Organizzazione del codice

La struttura attuale è cresciuta per sistemi e sottosistemi, ma necessita di un refactoring complessivo per raggruppare meglio implementazioni e test. La mappa completa dei file sorgente e delle destinazioni previste è nel documento operativo:

`.github/workflows/refactoring.md`

Al checkpoint del 7 novembre 2026, l'inventario di `src/` su `main` comprendeva 88 file Rust in 18 directory. L'audit iniziale delle righe fisiche ha individuato quattro file oltre 1.000 righe (nessuno oltre 1.500); la loro suddivisione è dettagliata in `.github/workflows/refactoring.md`. Questa è la baseline da cui è stato creato `refactor/src-domain-layout`. Durante il lavoro, verificare i file non ancora modificati rispetto a tale baseline e usare il branch di refactoring come fonte per i file già spostati o modificati. `main` deve rimanere intatto fino alla Pull Request finale.

Il refactoring deve organizzare il codice per dominio/sistema (per esempio `combat/summon/` per implementazione, sottosistemi e relativi test), mantenere chiari i confini tra moduli e lasciare in root soltanto i moduli di ingresso e gli aggregatori necessari. Non deve cambiare il comportamento del gioco.

## 4. Stato del sistema contenuti

Sono presenti identificatori con namespace, metadati, manifesti e dipendenze dei pacchetti, registro dei pacchetti, definizioni di scuole di danno, attributi, risorse, effetti, aure, abilità, talenti, database centrale, registrazione, lookup, validazione dei riferimenti e loader RON transazionale.

`ContentDatabase` è utilizzabile come `Resource` Bevy. Questo non significa che ogni pacchetto sia automaticamente caricato o che tutti i contenuti siano già collegati ai runtime.

### Creature e contenuti RON

`CreatureDefinition` contiene metadati e parametri runtime quali Health, Attack damage, Attack interval, Melee range, Aggro range, Leash range e Movement speed. Il database supporta registrazione, lookup, conteggio e validazione; il loader supporta `creatures` mantenendo compatibilità con i pack legacy che non includono il campo.

Il pack `content/packs/classic_1_12_1.ron` contiene la creatura Imp e contenuti dimostrativi utilizzati dai test e dalle integrazioni della Fase 3, inclusi Corruption e Summon.

Da ampliare nelle fasi successive: catalogo di abilità, talenti, creature, incontri e ricompense; eventuale ricaricamento dinamico; regole generali di sovrascrittura e precedenza tra pacchetti.

## 5. Stato del sistema di combattimento

### Statistiche, salute e risorse

Sono presenti `CombatStats`, `CombatAttributes`, `CombatLevel`, `HealthPool`, `CombatResources`, `ResourcePool`, `HealingRequest`, `HealingResolution`, `ResourceRequest` e `ResourceResolution`. Le pool supportano validazione, aggiornamento e applicazione delle operazioni. `HealthPool` resta l'unica fonte autorevole per la salute.

### Scuole di danno

Le sette scuole fondamentali sono Physical, Holy, Shadow, Nature, Frost, Arcane e Fire. Restano distinte dalle scuole composite. Il motore supporta genericamente le composite, ma il pack Classic attuale contiene soltanto tre definizioni: `Shadowfrost`, `Entropy` e `Frostfire`.

#### Catalogo progettuale delle scuole ibride a due componenti

La tabella conserva il catalogo progettuale completo. Le righe non presenti nel pack sono riferimenti per contenuti futuri e non implicano che siano già caricabili nel gioco.

| Scuola | Componenti | Stato nel pack Classic |
|---|---|---|
| `Holystrike` | Physical + Holy | Progettuale |
| `Shadowstrike` | Physical + Shadow | Progettuale |
| `Stormstrike` | Physical + Nature | Progettuale |
| `Icestrike` | Physical + Frost | Progettuale |
| `Spellstrike` | Physical + Arcane | Progettuale |
| `Firestrike` | Physical + Fire | Progettuale |
| `Twilight` | Holy + Shadow | Progettuale |
| `Anima` | Holy + Nature | Progettuale |
| `Holyfrost` | Holy + Frost | Progettuale |
| `Divine` | Holy + Arcane | Progettuale |
| `Radiant` | Holy + Fire | Progettuale |
| `Plague` | Shadow + Nature | Progettuale |
| `Shadowfrost` | Shadow + Frost | **Definita nel pack** |
| `Curse` | Shadow + Arcane | Progettuale |
| `Entropy` | Fire + Shadow | **Definita nel pack** |
| `Rainstorm` | Nature + Frost | Progettuale |
| `Astral` | Nature + Arcane | Progettuale |
| `Firestorm` | Nature + Fire | Progettuale |
| `Spellfrost` | Frost + Arcane | Progettuale |
| `Frostfire` | Frost + Fire | **Definita nel pack** |
| `Spellfire` | Arcane + Fire | Progettuale |

L'ordine dei componenti non cambia la scuola risultante: Physical + Nature e Nature + Physical identificano la stessa combinazione `Stormstrike`.

#### Esempi progettuali di scuole composite con tre o più componenti

| Scuola | Componenti |
|---|---|
| `Chaos` | Physical + Holy + Shadow + Nature + Frost + Arcane + Fire |
| `Cosmic` | Arcane + Holy + Nature + Shadow |
| `Elemental` | Fire + Frost + Nature |
| `Blood` | Physical + Nature + Shadow |
| `Chi` | Nature + Holy + Shadow |
| `Magical` | Arcane + Frost + Fire |

Queste composizioni sono esempi progettuali, non definizioni già presenti nel pack Classic.

#### Regole di mitigazione

Per ogni scuola composita si applica soltanto la mitigazione più alta tra quelle dei componenti; le mitigazioni non si sommano. Per combinazioni Physical + magia si confrontano le percentuali effettive di mitigazione fisica tramite armatura e mitigazione magica tramite resistenza. L'ordine dei componenti non cambia la mitigazione.

Formula dell'armatura:

`armor / (armor + 85 * attacker_level + 400)`

Mitigazione massima: 75%.

Formula della resistenza:

`min(0.75, max(resistance, 0) / (max(attacker_level, 20) * 5) * 0.75)`

La mitigazione è deterministica, senza tiri casuali per le mitigazioni parziali. A livello 60: 100 resistenza = 25%, 200 = 50%, 300 = 75%. La resistenza Holy dovrà essere supportata tramite un attributo dedicato `holy_resistance`.

Non aggiungere nuove combinazioni al pack senza una decisione esplicita dell'utente.

### Colpi critici, threat e displacement

Il calcolo dei colpi critici è nel modulo dedicato `damage/critical.rs`: riceve probabilità, moltiplicatore e tiro già generato. La probabilità deve rimanere tra 0 e 100 e il moltiplicatore deve essere almeno 1.

La threat supporta incrementi e riduzioni signed con clamp a zero e protezione da valori non validi. `PendingDisplacement` raccoglie richieste, risolve il vettore e applica lo spostamento attraverso il sistema fisico con collisioni; è integrato nel movimento dei giocatori, dei nemici e delle entità evocate.

## 6. Meccaniche provvisorie del combattimento

Queste meccaniche sono segnaposto e non rappresentano il bilanciamento definitivo.

### Ability 1 — Tank

- 15 danni.
- 20 risorse generate.
- Moltiplicatore threat 3.

### Ability 1 — DPS

- Con almeno 30 risorse: consuma 30 e infligge 40 danni.
- Altrimenti: 10 danni e 15 risorse generate.
- Moltiplicatore threat 1.

### Ability 1 — Healer

- Con almeno 20 risorse: consuma 20 e infligge 25 danni.
- Senza risorse sufficienti: l'abilità non viene eseguita.
- Moltiplicatore threat 1.

### Nemici

Valori provvisori: Aggro 20, Leash 40, portata melee 2,5, danno 5 e intervallo attacchi 2 secondi.

Non trasformare questi valori in regole definitive. Non applicare automaticamente critici, mitigazioni o nuove formule agli attacchi provvisori prima dell'integrazione nel sistema definitivo.

## 7. Stato delle fasi

### Fase 1 — Struttura dei contenuti

**Stato: COMPLETATA.**

Sono state realizzate le fondamenta data-driven: identificatori, metadati, manifesti, dipendenze, registro, database, registrazione, lookup, validazione e loader RON. Sono presenti definizioni per abilità, effetti, aure, risorse, scuole di danno, attributi, talenti e creature.

### Fase 2 — Fondamenta del combattimento

**Stato: COMPLETATA.**

Sono state realizzate le fondamenta per statistiche, attributi, livelli, salute, risorse, danno, mitigazioni, critici, cure, threat, eventi e displacement. Il sistema supporta le scuole di danno fondamentali e le definizioni composite. L'integrazione definitiva con tutte le abilità e gli attacchi viene sviluppata nelle fasi successive.

### Fase 3 — Sistema di esecuzione degli effetti

**Stato: COMPLETATA — 7 novembre 2026.**

È stato realizzato e verificato il runtime centralizzato che risolve le definizioni degli effetti e le applica alle entità ECS. La fase comprende:

- Risoluzione e calcolo della magnitudine degli effetti, scaling e validazione.
- Coda ed esecuzione ECS per danno, cure, risorse, aure e operazioni composte.
- Runtime delle aure con durata, stack, scope `Shared`/`PerCaster`, refresh e Pandemic.
- Effetti periodici, timer, tick, scadenze e scaling per stack.
- Eventi reattivi, dispatcher, sottoscrizioni e generazioni con protezione dai cicli incontrollati.
- Targeting AoE con validazione, relazioni tra entità, esclusione di bersagli non validi e deduplicazione.
- Registry ed esecuzione degli effetti personalizzati, inclusi test con definizioni caricate da RON.
- Integrazione di threat e displacement nel runtime previsto.
- Sistema Summon completo: catalogo creature, definizioni RON, spawn, fazione derivata dall'owner, statistiche, durata, despawn, morte, cleanup, IA, navigazione, targeting e combattimento per evocazioni alleate e nemiche.
- Test unitari, test end-to-end e audit di regressione per i sistemi sopra elencati.

Il pack Classic contiene Corruption come contenuto dimostrativo collegato al runtime periodico e le tre composite attualmente definite: Shadowfrost, Entropy e Frostfire. Le sette scuole fondamentali restano distinte dalle tre composite.

**Interrupt completo, stato di cast e channel non fanno parte della Fase 3:** sono rimandati alla Fase 4, dove potranno essere implementati insieme al runtime delle abilità.

La Fase 3 è stata chiusa con la Run #522: controllo preliminare Rust superato; 660 test superati e 0 falliti su Linux e Windows; avvio headless superato su entrambe le piattaforme. SHA verificato: `55e001ec7dc06f5d9bbb3107f3b5ef3f96354b38`.

### Fase 3.5 — Refactoring e riorganizzazione strutturale

**Stato: GRUPPI 1–6 COMPLETATI STRUTTURALMENTE E SOTTOPOSTI AD AUDIT STATICO. GRUPPO 7 IN CORSO: LA PRIMA CI (#536) È FALLITA DURANTE IL CONTROLLO DI FORMATTazione; L'ERRORE È STATO CORRETTO E PUBBLICATO, MA LA NUOVA CI SULLO SHA CORRETTO È ANCORA PENDENTE. COMPILAZIONE, SUITE DEI TEST E AVVIO HEADLESS NON SONO ANCORA STATI VERIFICATI SULLO SHA CORRETTO. LA VERIFICA FINALE È OBBLIGATORIA PRIMA DELLA FASE 4.**

Prima di sviluppare il sistema di esecuzione delle abilità, riorganizzare in modo pulito e coerente l'intera struttura dei sorgenti sotto `src/`.

Obiettivi:

- Raggruppare codice, sottosistemi e test in directory per dominio/responsabilità. Esempio: `src/combat/summon/` deve contenere tutto ciò che riguarda Summon e i relativi test.
- Dare una collocazione coerente anche ai sistemi aura, danno, effetti, nemici, giocatori, targeting, contenuti, navigazione e agli altri moduli presenti.
- Ridurre la dispersione dei file nella root di `src/` e rendere i confini tra moduli chiari.
- Valutare tutti i moduli e i test per responsabilità separabili, anche quando sono molto sotto le 1.000 righe. Raggruppare test simili per funzionalità e comportamento, separare gruppi che verificano responsabilità differenti, e mantenere fixture/helper condivisi in moduli comuni senza duplicazioni. La soglia di 1.000 righe fisiche è un limite di attenzione che richiede una valutazione, non il criterio minimo per suddividere. Le suddivisioni devono migliorare concretamente coesione e architettura, senza creare file minuscoli o frammenti artificiali. Oltre 1.000 righe effettuare comunque una valutazione e suddividere quando le responsabilità sono separabili; oltre 1.500 righe serve una motivazione documentata e approvazione esplicita.
- Aggiornare dichiarazioni `mod`, import, re-export, visibilità e percorsi dei test.
- Conservare tutti i file, i test e il comportamento attuale: questo passaggio è strutturale, non una modifica alle meccaniche di gioco.

Il documento operativo da seguire è:

`.github/workflows/refactoring.md`

Quel documento contiene l'inventario e la mappatura dei file, la struttura proposta, i passaggi operativi e le checklist. Va aggiornato durante il lavoro per segnare progressi, decisioni e nuovi percorsi. Prima di spostare ogni gruppo si deve verificare la baseline su `main` per i file non ancora toccati; per file già spostati o modificati, la versione di riferimento è quella presente su `refactor/src-domain-layout`.

Procedura obbligatoria:

1. Usare `main` soltanto come baseline stabile e verificare che il branch attivo sia `refactor/src-domain-layout`.
2. Eseguire tutti gli spostamenti, gli aggiornamenti di moduli e gli altri commit esclusivamente su `refactor/src-domain-layout`; non committare direttamente su `main`.
3. Seguire e, se necessario, correggere la mappa dei percorsi in `refactoring.md` prima degli spostamenti.
4. Riorganizzare un gruppo coerente alla volta, mantenendo tutti i file e test.
5. Valutare in tutti i domini ogni modulo e test per responsabilità separabili, indipendentemente dalle righe; suddividere quando c'è un vantaggio architetturale concreto. Usare 1.000 righe come soglia di attenzione e non come requisito minimo, raggruppare i test per funzione e riutilizzare fixture/helper condivisi. Evitare frammentazione artificiale; oltre 1.500 righe servono motivazione documentata e approvazione.
6. Aggiornare moduli, import, re-export, visibilità e percorsi dei test.
7. Controllare che non siano state introdotte modifiche funzionali involontarie.
8. Completare l'intera riorganizzazione e poi eseguire una sola verifica CI finale, salvo errori che rendano indispensabili controlli intermedi.
9. Verificare compilazione, test Linux e Windows e avvio headless; correggere ogni regressione prima di dichiarare conclusa la fase.
10. Aggiornare questo documento con la struttura definitiva, il commit e la CI finale.
11. Dopo audit e CI verde, preparare una Pull Request da `refactor/src-domain-layout` verso `main`. Integrare il branch soltanto dopo la revisione della Pull Request.

**Gate di avanzamento:** la Fase 4 non deve iniziare finché la Fase 3.5 non è completata, verificata e registrata nella roadmap. Non modificare il comportamento dei sistemi durante il refactoring senza una decisione esplicita dell'utente.

### Fase 4 — Sistema di esecuzione delle abilità

**Stato: NON INIZIATA — BLOCCATA FINO AL COMPLETAMENTO DELLA FASE 3.5.**

Dopo il refactoring, implementare il runtime che gestisce:

- Attivazione delle abilità.
- Selezione e validazione dei bersagli.
- Distanza e portata.
- Costi delle risorse.
- Tempi di recupero.
- Tempi di lancio e canalizzazioni.
- Interruzioni e stato di cast/channel.
- Abilità passive.
- Attivazioni basate su eventi.
- Prerequisiti e restrizioni di classe.
- Collegamento agli effetti definiti nei pacchetti.
- Sottoscrizioni automatiche agli eventi reattivi.

Le abilità dovranno essere data-driven quando supportate dal motore.

### Fase 5 — Contenuti originali Classic

**Stato: DA INIZIARE.**

Inserimento progressivo delle abilità originali Classic 1.12.1 per tutte le classi: abilità attive/passive, gradi, costi, requisiti, restrizioni, prerequisiti, effetti, scuole, cooldown e tempi di lancio. Poiché i personaggi partono al livello 60, usare i gradi massimi salvo eccezioni motivate. I contenuti devono essere definiti nei pack RON, non codificati singolarmente nel motore.

### Fase 6 — Talenti e configurazioni

**Stato: DA INIZIARE.**

Implementare i 27 alberi originali (tre per ciascuna delle nove classi), con gradi multipli, prerequisiti, effetti per grado, sblocco abilità, distribuzione punti, configurazioni complete da livello 60 e validazione. Il sistema deve supportare alberi e talenti personalizzati futuri.

### Fase 7 — Nemici e dungeon

**Stato: DA INIZIARE.**

Realizzare tipologie e comportamenti dei nemici, abilità ostili, aggro e threat, gruppi, boss e fasi, meccaniche degli incontri, difficoltà, creature e NPC modulari e interazioni con abilità ed effetti dei giocatori.

### Fase 8 — Equipaggiamento e ricompense

**Stato: DA INIZIARE.**

Implementare tipologie di equipaggiamento, statistiche, requisiti, restrizioni, affissi, inventario, loot table, ricompense dungeon e impatto dell'equipaggiamento su attributi e abilità.

### Fase 9 — Sistema cooperativo

**Stato: PARZIALMENTE AVVIATA.**

Fellowship, membri, ruoli e identificazione sono presenti nelle prime strutture. Restano da sviluppare gestione completa dei giocatori, interazioni e sinergie, attività condivise, gruppi e incontri, sincronizzazione, rete multiplayer, connessione/disconnessione e autorità dello stato di combattimento. La Fellowship non equivale a multiplayer di rete già implementato. Il gioco deve rimanere utilizzabile in single-player.

### Fase 10 — Espansione contenuti

**Stato: PIANIFICATA.**

Aggiungere classi, specializzazioni, abilità, talenti, scuole, risorse, effetti, statistiche e formule; gestire più pack, dipendenze, precedenze e sovrascritture. Classic 1.12.1 rimane il punto di partenza, non il limite del progetto.

### Fase 11 — Rifinitura e ottimizzazione

**Stato: PIANIFICATA.**

Ottimizzare performance ECS, allocazioni e risorse; completare UI, feedback audio/video, bilanciamento, test di regressione e rifinitura cooperativa.

## 8. CI e checkpoint verificati

### Run #450 — correzione storica di PendingDisplacement

- Run ID: `42356059517`.
- Commit: `aabedbb03cc071857a19370006e6c3bff5ebf734` (`Update movement.rs`).
- Esito: success su controllo preliminare Rust, Linux e Windows.
- Correzione: import di `PendingDisplacement` spostato da `super` a `crate::combat` in `src/player/movement.rs`.
- Run: https://github.com/konakona185-gif/wow-friendship-crawler/actions/runs/42356059517

Le Run 446–448 erano fallite per l'import non risolto; la Run 449 era stata annullata dopo un tentativo intermedio errato. La Run #450 è il checkpoint storico che conferma la correzione.

### Run #522 — CI finale della Fase 3

- Run ID: `42401651307`.
- Commit/SHA: `55e001ec7dc06f5d9bbb3107f3b5ef3f96354b38` (`Update main.rs`).
- Esito finale: **success**.
- Controllo preliminare Rust: success.
- Linux: success, 660 test superati, 0 falliti e avvio headless superato.
- Windows: success, 660 test superati, 0 falliti e avvio headless superato.
- Run: https://github.com/konakona185-gif/wow-friendship-crawler/actions/runs/42401651307

Questa è la CI usata per dichiarare completata la Fase 3. Una CI è verde soltanto quando `conclusion = success` sulla run associata allo SHA esatto. `cancelled`, `failure`, `in_progress` e `pending` non sono esiti verdi.

### Run #536 — prima verifica tecnica della Fase 3.5

- Run ID: `42504899025`.
- Branch: `refactor/src-domain-layout`.
- Commit/SHA testato: `0cfcf2a61fff40500c7f9aac7ebec7c50cae7c77`.
- Evento: `workflow_dispatch`.
- Esito finale: **failure**.
- Il job preliminare Rust è fallito durante `cargo fmt --all -- --check`.
- Errore: `expected item after attributes` in `src/combat/effects/dispatcher/tests/reactive_runtime_tests.rs`, riga 488.
- Causa: una direttiva `#[cfg(test)]` era rimasta orfana dopo lo spostamento della dichiarazione `mod area_tests` nel modulo parent.
- I job di compilazione e test Windows e Linux sono stati saltati perché dipendenti dal controllo preliminare.
- Compilazione, test release e avvio headless non sono stati verificati in questa esecuzione.
- Run: https://github.com/konakona185-gif/wow-friendship-crawler/actions/runs/42504899025

### Correzione successiva alla Run #536

- Correzione applicata: rimossa esclusivamente la direttiva `#[cfg(test)]` orfana da `src/combat/effects/dispatcher/tests/reactive_runtime_tests.rs`.
- La dichiarazione `mod area_tests` rimane nel modulo parent `tests/mod.rs`, già protetto da `#[cfg(test)]`.
- Gli 11 test di `reactive_runtime_tests.rs` e le suite sibling sono stati preservati.
- Commit della correzione: `b69aefcf7efe844d8c9bf32227bf04ec6cfca2ba`.
- Messaggio: `Remove unnecessary lines from reactive_runtime_tests.rs`.
- Il commit modifica esclusivamente `src/combat/effects/dispatcher/tests/reactive_runtime_tests.rs`, rimuovendo la direttiva orfana e la riga vuota finale.
- La modifica è stata verificata leggendo il file aggiornato e confrontando il diff del commit su GitHub.
- La correzione non è ancora stata verificata tramite una nuova esecuzione GitHub Actions.
- Formattazione, compilazione, test Linux/Windows e avvio headless restano da verificare sul nuovo SHA.

## 9. Regole operative di sviluppo

### File e modifiche

- Prima di intervenire verificare che il branch attivo sia `refactor/src-domain-layout`.
- Per i file non ancora toccati, confrontare il contenuto completo con la baseline di `main`; per i file già spostati o modificati, recuperare sempre la versione corrente dal branch di refactoring.
- Non ricostruire file dalla memoria o da vecchi messaggi.
- Dopo ogni conferma `fatto`, verificare che la modifica sia presente nel branch di refactoring, non su `main`.
- Per modifiche strutturali o multilinea fornire il file completo aggiornato; se l'utente chiede il file completo, fornirlo integralmente.
- Per modifiche semplici di una sola riga indicare percorso e testo esatto da sostituire.
- Evitare istruzioni ambigue sulla posizione delle modifiche.

### Struttura e qualità

- Nomi di file e directory devono descrivere la responsabilità.
- Ogni spostamento richiede controllo di moduli, import, test, re-export e visibilità.
- Preservare tutte le funzionalità e i test esistenti.
- Non aggiungere commenti al codice (`//` o `#`), salvo attributi Rust necessari.
- Preferire sistemi modulari, riutilizzabili e data-driven; evitare duplicazioni e frammentazioni artificiali.
- Valutare la coesione di ogni file Rust sotto `src/`, inclusi i test, indipendentemente dal numero di righe. La soglia di **1.000 righe fisiche** (comprese righe vuote e commenti) è un limite di attenzione che richiede una valutazione, non il criterio minimo per suddividere.
- Suddividere file di qualunque dimensione quando responsabilità distinte possono essere organizzate in unità coerenti con un vantaggio architetturale concreto. Oltre 1.000 righe la valutazione è obbligatoria; oltre 1.500 righe serve una motivazione tecnica documentata e approvazione esplicita.
- Per i test, raggruppare i casi simili per funzionalità e comportamento e separare i gruppi che verificano responsabilità diverse; mantenere fixture e helper condivisi in moduli comuni ed evitare duplicazioni. Non creare file minuscoli o frammenti senza vantaggio concreto.
- **1.500 righe è il limite eccezionale**: un file oltre tale soglia richiede motivazione tecnica documentata e approvazione esplicita.
- Prima della chiusura della Fase 3.5 eseguire un audit dimensionale completo e registrare tutte le eccezioni.
- I test devono essere suddivisi per aree funzionali; fixture e helper condivisi possono rimanere nel rispettivo `tests/mod.rs`.
- Spiegare le conseguenze delle modifiche alle meccaniche e ottenere approvazione prima di applicarle.

### CI e commit

- Tutti i commit del refactoring devono essere creati su `refactor/src-domain-layout`; `main` non va modificato direttamente durante la Fase 3.5.
- Raggruppare modifiche compatibili prima di avviare nuove verifiche; evitare run intermedie inutili.
- Non è necessario attendere una CI verde per ogni singolo file se è sicuro proseguire.
- Dichiarare una CI superata solo dopo aver verificato l'esito e lo SHA esatto del branch interessato.
- Non confondere il risultato di un commit precedente con quello attuale.
- L'unione in `main` avviene solo tramite Pull Request dopo audit e CI finale verde.

### Continuità

Questa roadmap è la memoria progettuale primaria. Prima di ogni sessione verificare repository, commit, file interessati e CI corrente. Aggiornare la roadmap dopo milestone o cambiamenti architetturali significativi, non dopo ogni commit.

Il documento operativo del refactoring è `.github/workflows/refactoring.md` e va mantenuto sincronizzato durante la Fase 3.5.

Durante queste sessioni concentrarsi esclusivamente sul progetto WoW, non sul progetto Godot.

## 10. Prossimi passaggi immediati

1. La Fase 3 è completata sulla revisione `55e001ec7dc06f5d9bbb3107f3b5ef3f96354b38`, verificata dalla Run #522.
2. Proseguire esclusivamente sul branch `refactor/src-domain-layout`, creato dallo SHA `454874a358a18e7e565043d797e3c42ddc46323f`; non modificare direttamente `main`.
3. I Gruppi 1–6 della Fase 3.5 sono completati strutturalmente e sottoposti ad audit statico; la mappatura finale comprende 135 file e il confronto statico registra 660 test (656 nomi distinti), senza differenze rispetto alla baseline.
4. Il Gruppo 7 è in corso. La Run #536, eseguita sullo SHA `0cfcf2a61fff40500c7f9aac7ebec7c50cae7c77`, è fallita durante `cargo fmt --all -- --check` per una direttiva `#[cfg(test)]` orfana. I job di compilazione e test Linux/Windows sono stati saltati perché il controllo preliminare è fallito.
5. La direttiva orfana è stata rimossa e la correzione è stata pubblicata sul branch con il commit `b69aefcf7efe844d8c9bf32227bf04ec6cfca2ba`. La correzione non è ancora stata verificata da una nuova esecuzione CI.
6. Il controllo `cargo fmt --all -- --check` in `.github/workflows/build.yml` è bloccante: un errore fa fallire il job. La prima esecuzione ha confermato che il controllo blocca correttamente il workflow.
7. Eseguire nuovamente la CI finale Linux/Windows/headless sullo SHA aggiornato, verificando formattazione, compilazione, test, avvio headless, artifact ed esatto SHA della run.
8. Registrare in questa roadmap struttura definitiva, SHA e risultati CI dopo che le verifiche saranno effettivamente completate.
9. Solo dopo audit e CI verde preparare una Pull Request verso `main`; integrare il branch soltanto dopo revisione e approvazione esplicita dell'utente.
10. Solo dopo la chiusura verificata della Fase 3.5 iniziare la Fase 4.

**Non iniziare la Fase 4 prima del completamento del refactoring.**

## 11. Posizione dei documenti e fonti di verità

Questa roadmap si trova in:

`.github/workflows/benilla-classic-pipeline-revised-v3.md`

Il piano operativo del refactoring si trova in:

`.github/workflows/refactoring.md`

Entrambi sono collocati nella cartella `.github/workflows/`, insieme al workflow YAML. Non spostarli durante il refactoring dei sorgenti, salvo decisione esplicita.

In caso di discrepanza:

- `main` rappresenta la baseline stabile del progetto e non deve essere modificato direttamente durante la Fase 3.5.
- `refactor/src-domain-layout` è la fonte di verità per lo stato corrente dei lavori di riorganizzazione.
- Le decisioni progettuali approvate e registrate qui stabiliscono il comportamento desiderato.
- `refactoring.md` governa la mappatura e l'avanzamento della riorganizzazione strutturale.
- I log CI stabiliscono gli errori effettivamente verificatisi sullo SHA testato.

L'obiettivo è costruire un sistema di gioco modulare, coerente, testabile ed espandibile, con i contenuti Classic come base e senza limiti artificiali alle estensioni future.
