# Refactoring – piano operativo permanente

> Documento di lavoro per la riorganizzazione strutturale del codice Rust sotto `src/`.
> Questo file va mantenuto nel repository come riferimento vivo: si aggiornano qui stato, decisioni, mappature e verifiche man mano che il refactoring procede.

## 1. Obiettivo

Riorganizzare `src/` per domini e sottosistemi, in modo che implementazioni, moduli di supporto e test correlati siano collocati insieme. La struttura deve rimanere leggibile e pronta per l'espansione futura del progetto.

Il refactoring è **strutturale**: durante gli spostamenti non si devono cambiare deliberatamente le meccaniche di gioco, le regole di combattimento, i dati di contenuto o il comportamento osservabile. Eventuali correzioni funzionali emerse durante il lavoro vanno registrate separatamente e affrontate solo dopo aver isolato il puro riordino, salvo che siano indispensabili per compilare dopo uno spostamento.

## 2. Regole non negoziabili

1. Non eliminare, accorpare o riscrivere test solo per rendere più semplice lo spostamento.
2. Ogni file Rust esistente deve avere una destinazione esplicita. Il registro in questo documento copre tutti gli **88 file** trovati nell'inventario iniziale.
3. Prima di spostare un modulo, controllare `mod`, `pub mod`, `pub use`, `use super::*`, `#[path = ...]`, `include_str!` e ogni riferimento al suo percorso.
4. Mantenere le API pubbliche e `pub(crate)` già usate dagli altri sistemi, tramite re-export espliciti quando cambia la gerarchia dei moduli.
5. I test devono rimanere compilati nelle stesse configurazioni in cui lo sono oggi. Un test non è completato solo perché il file è stato spostato.
6. Non cambiare contemporaneamente architettura e comportamento: prima spostamento e wiring, poi eventuali refactor interni separati.
7. Procedere per gruppi piccoli e coerenti, ma accumulare le modifiche correlate prima della verifica CI, evitando run intermedie non necessarie.
8. Fornire file completi quando una modifica interessa più punti dello stesso file o la struttura del modulo; per cambi semplici e isolati è ammessa una sostituzione puntuale.
9. Non iniziare la Fase 4 del progetto finché questo refactoring non è completato e verificato.

## 3. Baseline e inventario

- Repository: `konakona185-gif/wow-friendship-crawler`
- Branch di riferimento esaminato: `main`
- File Rust censiti sotto `src/`: **88**
- Directory censite sotto `src/`: **18**
- Dimensione complessiva iniziale: **913.840 byte**
- Percorsi sorgente mappati: **88/88**
- Destinazioni duplicate nella proposta: **0**
- Nuovi moduli aggregatori di dominio previsti sotto `combat/`: **6** (`aura`, `core`, `effects`, `enemy`, `player`, `targeting`)
- La scansione GitHub dell'albero iniziale risultava completa (`truncated: false`).

La CI di riferimento precedente al refactoring è la Run **#522**, con 660 test superati su Linux e Windows e avvio headless superato. Durante il refactoring il conteggio dei test deve essere confrontato con il baseline effettivo: il risultato atteso è nessun test perso e nessuna regressione. Prima di iniziare gli spostamenti, verificare comunque che `main` sia ancora verde e registrare SHA e numero della run effettivamente usati come base.

## 4. Modalità di lavoro e gestione del documento

Questo file è il registro operativo. Per ogni gruppo di lavoro:

- segnare lo stato come `DA FARE`, `IN CORSO`, `COMPLETATO` o `BLOCCATO`;
- annotare i percorsi completati e le eventuali eccezioni;
- registrare gli eventuali cambi alla struttura proposta prima di applicarli;
- aggiornare il numero di file verificati e il risultato delle prove;
- non cancellare la cronologia delle decisioni: aggiungere una breve nota datata nella sezione finale.

Il piano può essere aggiornato quando l'ispezione del codice dimostra che una destinazione è inappropriata. In quel caso si aggiorna prima la tabella vecchio percorso → nuovo percorso e poi si sposta il file. Non lasciare mappature obsolete.

## 5. Fasi operative

### Fase 0 – Preparazione e salvaguardia

**Stato iniziale: DA FARE**

- [ ] Verificare il branch `main`, lo SHA HEAD e lo stato della CI.
- [ ] Verificare che il working tree locale sia pulito prima di iniziare.
- [ ] Creare un branch dedicato, ad esempio `refactor/src-domain-layout`, partendo dallo SHA verificato.
- [ ] Conservare questo documento nella root del repository, allo stesso livello di `Cargo.toml`.
- [ ] Conservare l'inventario degli 88 file e usare la tabella della sezione 7 come checklist.
- [ ] Confermare che il branch di lavoro non contenga modifiche funzionali non pertinenti.

### Fase 1 – Conversione dei moduli di primo livello

**Stato iniziale: DA FARE**

Convertire i file che rappresentano moduli principali in directory con `mod.rs`, senza cambiare i nomi logici dei moduli importati da `main.rs`:

- [ ] `camera.rs` → `camera/mod.rs`
- [ ] `character.rs` → `character/mod.rs`
- [ ] `combat.rs` → `combat/mod.rs`
- [ ] `content.rs` → `content/mod.rs`
- [ ] `fellowship.rs` → `fellowship/mod.rs`
- [ ] `interaction.rs` → `interaction/mod.rs`
- [ ] `navigation.rs` → `navigation/mod.rs`
- [ ] `player.rs` → `player/mod.rs`
- [ ] Integrare `enemy_navigation.rs` come sottosistema `navigation/enemy/mod.rs`.

Controlli specifici:

- [ ] Aggiornare `main.rs` se cambia il punto di accesso al plugin di navigazione nemica; idealmente `main.rs` importa `EnemyNavigationPlugin` da `navigation` e non mantiene un modulo top-level separato `enemy_navigation`.
- [ ] Assicurarsi che `mod.rs` continui a esporre gli stessi simboli pubblici richiesti da `main.rs` e dagli altri domini.
- [ ] Verificare che la conversione `file.rs` → `directory/mod.rs` non abbia cambiato la risoluzione dei moduli figli.

### Fase 2 – Riordino del dominio combat

**Stato iniziale: DA FARE**

Spostare i sistemi di combattimento nelle aree proposte nella sezione 6 e nell'albero della sezione 7.

- [ ] `aura/`: esecuzione, runtime, sistemi e refresh delle aure.
- [ ] `core/`: eventi di combattimento, healing, health pool, risorse e statistiche.
- [ ] `damage/`: critici e mitigazione.
- [ ] `death/` e `displacement/`: relativi sistemi e test.
- [ ] `effects/`: custom effects, dispatcher, execution, magnitude, resolution, targeting e runtime.
- [ ] `enemy/`: AI e attacchi dei nemici.
- [ ] `player/`: attacchi del personaggio giocante.
- [ ] `summon/`: entità evocate, AI, attacchi, lifecycle e test di integrazione.
- [ ] `targeting/`: indicatore e selezione del bersaglio.
- [ ] Aggiornare `combat/mod.rs` e i re-export per mantenere stabile l'interfaccia esterna.

**Controlli particolarmente importanti per `summon/`:**

- [ ] Mantenere `SummonedCreature`, `AllySummon`, `AllySummonAi`, `SummonFaction` e gli altri tipi pubblici disponibili tramite `crate::combat` dove sono già utilizzati.
- [ ] Mantenere accessibile `SummonCombatProfile` con la visibilità `pub(crate)` prevista.
- [ ] Controllare gli import tra `summon/`, `enemy/` e `combat/core`; non assumere che `use super::*` continui a riferirsi agli stessi simboli dopo lo spostamento.
- [ ] Spostare il test di integrazione in `summon/tests/integration.rs` e aggiornare esplicitamente il percorso del modulo test in `combat/mod.rs`, oppure adottare una dichiarazione equivalente verificata in compilazione.
- [ ] Verificare che le regole di fazione delle summon e la distinzione tra `Character`, `Enemy` e `AllySummon` restino invariate.

### Fase 3 – Riordino di content, character, player e navigation

**Stato iniziale: DA FARE**

- [ ] Spostare `content.rs` e `content/database.rs` nelle rispettive directory `mod.rs`, mantenendo i modelli e le API esistenti.
- [ ] Riordinare i test del database in `content/database/tests/` senza perdere i test di aura, classic pack, reference e registration.
- [ ] Riordinare `character/` e il relativo test.
- [ ] Riordinare `player/` mantenendo input, movement e movement helpers.
- [ ] Accorpare la navigazione nemica sotto `navigation/enemy/` e mantenere il plugin registrato una sola volta.
- [ ] Controllare tutti gli import cross-domain in `main.rs`, `combat`, `player`, `navigation` e `interaction`.

### Fase 4 – Audit completo di moduli, import e test

**Stato iniziale: DA FARE**

- [ ] Cercare ogni dichiarazione `mod` e verificare che punti al file effettivo.
- [ ] Cercare ogni attributo `#[path = ...]` e aggiornare i percorsi spostati.
- [ ] Controllare ogni `use super::*`: dopo il riordino `super` può indicare un modulo diverso.
- [ ] Controllare i re-export `pub use` e `pub(crate) use`; non allargare la visibilità senza motivo.
- [ ] Controllare `include_str!`, `include_bytes!`, path relativi e riferimenti a file non Rust.
- [ ] Verificare che i test esterni e quelli inline siano tutti ancora inclusi con `#[cfg(test)]`.
- [ ] Cercare riferimenti testuali ai vecchi percorsi, inclusi workflow o script se presenti.
- [ ] Confrontare il numero dei test prima e dopo; spiegare ogni differenza anziché ignorarla.

### Fase 5 – Verifica tecnica e chiusura

**Stato iniziale: DA FARE**

- [ ] Eseguire formattazione Rust (`cargo fmt --all -- --check`) e correggere eventuali formattazioni introdotte dagli spostamenti.
- [ ] Eseguire i controlli di compilazione e test locali disponibili, senza dichiarare la fase conclusa prima dei risultati.
- [ ] Eseguire una sola verifica CI completa dopo aver raggruppato tutte le modifiche strutturali, salvo blocchi che richiedano una diagnosi anticipata.
- [ ] Verificare entrambi i job Linux e Windows e l'avvio headless.
- [ ] Verificare che non siano diminuiti i test rispetto al baseline senza una motivazione documentata.
- [ ] Ispezionare il diff per assicurarsi che i cambiamenti siano principalmente spostamenti, aggiornamenti dei moduli e import.
- [ ] Aggiornare la roadmap principale indicando il refactoring come fase preparatoria completata.
- [ ] Registrare qui SHA finale, run CI, numero test e data di completamento.

## 6. Decisioni architetturali della struttura proposta

- `src/main.rs` resta il punto d'ingresso e non viene spostato.
- Ogni dominio top-level ha una directory con `mod.rs`.
- `combat/` viene diviso per responsabilità; non si crea un unico contenitore generico con file eterogenei.
- I sistemi evocati vengono raggruppati in `combat/summon/`; il relativo test di integrazione sta nello stesso dominio.
- I test già separati vengono raccolti in directory `tests/` del sistema proprietario.
- I test attualmente inline nei file sorgente non vengono estratti automaticamente in questa prima passata: si mantengono per ridurre il rischio e si valuta una loro eventuale estrazione separatamente.
- I file `.ron`, `Cargo.toml` e workflow CI non vengono spostati da questa attività.
- Le directory `combat/aura`, `combat/core`, `combat/effects`, `combat/enemy`, `combat/player` e `combat/targeting` avranno un `mod.rs` aggregatore ciascuna.

## 7. Struttura di destinazione e registro completo dei file

L'albero seguente rappresenta la destinazione pianificata. La tabella immediatamente successiva è la checklist autorevole: ogni file sorgente deve comparire una volta, e una sola, nella colonna delle destinazioni.

## Albero proposto

```text
src/
├── camera/
│   └── mod.rs
├── character/
│   ├── tests/
│   │   └── mod.rs
│   ├── build.rs
│   ├── class.rs
│   ├── mod.rs
│   └── race.rs
├── combat/
│   ├── aura/
│   │   ├── runtime/
│   │   │   ├── tests/
│   │   │   │   └── mod.rs
│   │   │   ├── mod.rs
│   │   │   └── refresh.rs
│   │   ├── execution.rs
│   │   ├── mod.rs
│   │   └── systems.rs
│   ├── core/
│   │   ├── events.rs
│   │   ├── healing.rs
│   │   ├── health.rs
│   │   ├── mod.rs
│   │   ├── resources.rs
│   │   └── stats.rs
│   ├── damage/
│   │   ├── critical.rs
│   │   ├── mitigation.rs
│   │   └── mod.rs
│   ├── death/
│   │   ├── tests/
│   │   │   └── mod.rs
│   │   └── mod.rs
│   ├── displacement/
│   │   └── mod.rs
│   ├── effects/
│   │   ├── dispatcher/
│   │   │   ├── tests/
│   │   │   │   ├── area_tests.rs
│   │   │   │   └── mod.rs
│   │   │   └── mod.rs
│   │   ├── runtime/
│   │   │   ├── tests/
│   │   │   │   ├── aura_tests.rs
│   │   │   │   ├── corruption_tests.rs
│   │   │   │   ├── custom_effect_tests.rs
│   │   │   │   ├── displacement_tests.rs
│   │   │   │   ├── mod.rs
│   │   │   │   └── periodic_tests.rs
│   │   │   └── mod.rs
│   │   ├── custom.rs
│   │   ├── execution.rs
│   │   ├── magnitude.rs
│   │   ├── mod.rs
│   │   ├── resolution.rs
│   │   └── targeting.rs
│   ├── enemy/
│   │   ├── ai/
│   │   │   ├── tests/
│   │   │   │   └── mod.rs
│   │   │   └── mod.rs
│   │   ├── attacks/
│   │   │   ├── tests/
│   │   │   │   └── mod.rs
│   │   │   └── mod.rs
│   │   └── mod.rs
│   ├── player/
│   │   ├── attacks/
│   │   │   ├── tests/
│   │   │   │   └── mod.rs
│   │   │   └── mod.rs
│   │   └── mod.rs
│   ├── summon/
│   │   ├── tests/
│   │   │   └── integration.rs
│   │   ├── ai.rs
│   │   ├── attacks.rs
│   │   ├── lifecycle.rs
│   │   └── mod.rs
│   ├── targeting/
│   │   ├── indicator/
│   │   │   ├── tests/
│   │   │   │   └── mod.rs
│   │   │   └── mod.rs
│   │   ├── selection/
│   │   │   ├── tests/
│   │   │   │   └── mod.rs
│   │   │   └── mod.rs
│   │   └── mod.rs
│   └── mod.rs
├── content/
│   ├── database/
│   │   ├── tests/
│   │   │   ├── aura_tests.rs
│   │   │   ├── classic_pack_tests.rs
│   │   │   ├── mod.rs
│   │   │   ├── reference_tests.rs
│   │   │   └── registration_tests.rs
│   │   ├── lookup.rs
│   │   ├── mod.rs
│   │   ├── registration.rs
│   │   └── validation.rs
│   ├── ability.rs
│   ├── aura.rs
│   ├── combat_attribute.rs
│   ├── creature.rs
│   ├── damage_school.rs
│   ├── effect.rs
│   ├── id.rs
│   ├── loader.rs
│   ├── metadata.rs
│   ├── mod.rs
│   ├── pack.rs
│   ├── registry.rs
│   ├── resource.rs
│   └── talent.rs
├── fellowship/
│   └── mod.rs
├── interaction/
│   └── mod.rs
├── navigation/
│   ├── enemy/
│   │   ├── mod.rs
│   │   ├── movement.rs
│   │   ├── movement_helpers.rs
│   │   └── targets.rs
│   └── mod.rs
└── player/
    ├── input.rs
    ├── mod.rs
    ├── movement.rs
    └── movement_helpers.rs
```

## Registro completo degli 88 file (percorso attuale → destinazione)

| Percorso attuale | Percorso proposto |
|---|---|
| `src/camera.rs` | `src/camera/mod.rs` |
| `src/character.rs` | `src/character/mod.rs` |
| `src/character/build.rs` | `src/character/build.rs` |
| `src/character/class.rs` | `src/character/class.rs` |
| `src/character/race.rs` | `src/character/race.rs` |
| `src/character/tests.rs` | `src/character/tests/mod.rs` |
| `src/combat.rs` | `src/combat/mod.rs` |
| `src/combat/aura_execution.rs` | `src/combat/aura/execution.rs` |
| `src/combat/aura_runtime.rs` | `src/combat/aura/runtime/mod.rs` |
| `src/combat/aura_runtime/refresh.rs` | `src/combat/aura/runtime/refresh.rs` |
| `src/combat/aura_runtime/tests.rs` | `src/combat/aura/runtime/tests/mod.rs` |
| `src/combat/aura_systems.rs` | `src/combat/aura/systems.rs` |
| `src/combat/custom_effects.rs` | `src/combat/effects/custom.rs` |
| `src/combat/damage.rs` | `src/combat/damage/mod.rs` |
| `src/combat/damage/critical.rs` | `src/combat/damage/critical.rs` |
| `src/combat/damage/mitigation.rs` | `src/combat/damage/mitigation.rs` |
| `src/combat/death.rs` | `src/combat/death/mod.rs` |
| `src/combat/death/tests.rs` | `src/combat/death/tests/mod.rs` |
| `src/combat/displacement.rs` | `src/combat/displacement/mod.rs` |
| `src/combat/effect_dispatcher.rs` | `src/combat/effects/dispatcher/mod.rs` |
| `src/combat/effect_dispatcher/tests.rs` | `src/combat/effects/dispatcher/tests/mod.rs` |
| `src/combat/effect_dispatcher/tests/area_tests.rs` | `src/combat/effects/dispatcher/tests/area_tests.rs` |
| `src/combat/effect_execution.rs` | `src/combat/effects/execution.rs` |
| `src/combat/effect_magnitude.rs` | `src/combat/effects/magnitude.rs` |
| `src/combat/effect_resolution.rs` | `src/combat/effects/resolution.rs` |
| `src/combat/effect_runtime.rs` | `src/combat/effects/runtime/mod.rs` |
| `src/combat/effect_runtime/aura_tests.rs` | `src/combat/effects/runtime/tests/aura_tests.rs` |
| `src/combat/effect_runtime/corruption_tests.rs` | `src/combat/effects/runtime/tests/corruption_tests.rs` |
| `src/combat/effect_runtime/custom_effect_tests.rs` | `src/combat/effects/runtime/tests/custom_effect_tests.rs` |
| `src/combat/effect_runtime/displacement_tests.rs` | `src/combat/effects/runtime/tests/displacement_tests.rs` |
| `src/combat/effect_runtime/periodic_tests.rs` | `src/combat/effects/runtime/tests/periodic_tests.rs` |
| `src/combat/effect_runtime/tests.rs` | `src/combat/effects/runtime/tests/mod.rs` |
| `src/combat/effect_targeting.rs` | `src/combat/effects/targeting.rs` |
| `src/combat/enemy_ai.rs` | `src/combat/enemy/ai/mod.rs` |
| `src/combat/enemy_ai/tests.rs` | `src/combat/enemy/ai/tests/mod.rs` |
| `src/combat/enemy_attacks.rs` | `src/combat/enemy/attacks/mod.rs` |
| `src/combat/enemy_attacks/tests.rs` | `src/combat/enemy/attacks/tests/mod.rs` |
| `src/combat/events.rs` | `src/combat/core/events.rs` |
| `src/combat/healing.rs` | `src/combat/core/healing.rs` |
| `src/combat/health.rs` | `src/combat/core/health.rs` |
| `src/combat/player_attacks.rs` | `src/combat/player/attacks/mod.rs` |
| `src/combat/player_attacks/tests.rs` | `src/combat/player/attacks/tests/mod.rs` |
| `src/combat/resources.rs` | `src/combat/core/resources.rs` |
| `src/combat/stats.rs` | `src/combat/core/stats.rs` |
| `src/combat/summon_ai.rs` | `src/combat/summon/ai.rs` |
| `src/combat/summon_attacks.rs` | `src/combat/summon/attacks.rs` |
| `src/combat/summon_integration_tests.rs` | `src/combat/summon/tests/integration.rs` |
| `src/combat/summon_lifecycle.rs` | `src/combat/summon/lifecycle.rs` |
| `src/combat/summons.rs` | `src/combat/summon/mod.rs` |
| `src/combat/target_indicator.rs` | `src/combat/targeting/indicator/mod.rs` |
| `src/combat/target_indicator/tests.rs` | `src/combat/targeting/indicator/tests/mod.rs` |
| `src/combat/target_selection.rs` | `src/combat/targeting/selection/mod.rs` |
| `src/combat/target_selection/tests.rs` | `src/combat/targeting/selection/tests/mod.rs` |
| `src/content.rs` | `src/content/mod.rs` |
| `src/content/ability.rs` | `src/content/ability.rs` |
| `src/content/aura.rs` | `src/content/aura.rs` |
| `src/content/combat_attribute.rs` | `src/content/combat_attribute.rs` |
| `src/content/creature.rs` | `src/content/creature.rs` |
| `src/content/damage_school.rs` | `src/content/damage_school.rs` |
| `src/content/database.rs` | `src/content/database/mod.rs` |
| `src/content/database/lookup.rs` | `src/content/database/lookup.rs` |
| `src/content/database/registration.rs` | `src/content/database/registration.rs` |
| `src/content/database/tests.rs` | `src/content/database/tests/mod.rs` |
| `src/content/database/tests/aura_tests.rs` | `src/content/database/tests/aura_tests.rs` |
| `src/content/database/tests/classic_pack_tests.rs` | `src/content/database/tests/classic_pack_tests.rs` |
| `src/content/database/tests/reference_tests.rs` | `src/content/database/tests/reference_tests.rs` |
| `src/content/database/tests/registration_tests.rs` | `src/content/database/tests/registration_tests.rs` |
| `src/content/database/validation.rs` | `src/content/database/validation.rs` |
| `src/content/effect.rs` | `src/content/effect.rs` |
| `src/content/id.rs` | `src/content/id.rs` |
| `src/content/loader.rs` | `src/content/loader.rs` |
| `src/content/metadata.rs` | `src/content/metadata.rs` |
| `src/content/pack.rs` | `src/content/pack.rs` |
| `src/content/registry.rs` | `src/content/registry.rs` |
| `src/content/resource.rs` | `src/content/resource.rs` |
| `src/content/talent.rs` | `src/content/talent.rs` |
| `src/enemy_navigation.rs` | `src/navigation/enemy/mod.rs` |
| `src/enemy_navigation/movement.rs` | `src/navigation/enemy/movement.rs` |
| `src/enemy_navigation/movement_helpers.rs` | `src/navigation/enemy/movement_helpers.rs` |
| `src/enemy_navigation/targets.rs` | `src/navigation/enemy/targets.rs` |
| `src/fellowship.rs` | `src/fellowship/mod.rs` |
| `src/interaction.rs` | `src/interaction/mod.rs` |
| `src/main.rs` | `src/main.rs` |
| `src/navigation.rs` | `src/navigation/mod.rs` |
| `src/player.rs` | `src/player/mod.rs` |
| `src/player/input.rs` | `src/player/input.rs` |
| `src/player/movement.rs` | `src/player/movement.rs` |
| `src/player/movement_helpers.rs` | `src/player/movement_helpers.rs` |

## Regole architetturali

1. `src/main.rs` resta il punto d'ingresso; gli altri moduli di primo livello diventano directory con `mod.rs`.
2. Ogni dominio ha una propria directory: personaggi, combattimento, contenuti, navigazione, giocatore, fellowship e interazione.
3. Dentro `combat/`, i sottosistemi sono separati per responsabilità: `aura`, `damage`, `death`, `displacement`, `effects`, `enemy`, `player`, `summon`, `targeting` e `core`.
4. I test esterni sono raccolti nella directory `tests/` del rispettivo sistema. I test attualmente inline nei file restano per il momento dove sono: estrarli sarebbe un intervento ulteriore, non un semplice spostamento.
5. I sei `mod.rs` aggiuntivi sotto `combat/` sono aggregatori nuovi, necessari per esporre ordinatamente i sottosistemi: `aura`, `core`, `effects`, `enemy`, `player` e `targeting`.
6. I file di contenuto `.ron` e la configurazione Cargo/workflow non vengono spostati in questa riorganizzazione.

## 8. Registro di avanzamento

| Area | Stato | Note / commit |
|---|---|---|
| Fase 0 – Preparazione | DA FARE | |
| Fase 1 – Moduli top-level | DA FARE | |
| Fase 2 – Dominio combat | DA FARE | |
| Fase 3 – Altri domini | DA FARE | |
| Fase 4 – Audit moduli/import/test | DA FARE | |
| Fase 5 – Verifica e chiusura | DA FARE | |

### Registro delle decisioni e delle eccezioni

| Data | Decisione / modifica al piano | Motivazione | Stato |
|---|---|---|---|
| 2026-11-07 | Creato il piano iniziale con inventario completo di 88 file. | Preparare la riorganizzazione prima di iniziare la Fase 4 del progetto. | REGISTRATO |

## 9. Criteri di completamento

Il refactoring è concluso soltanto quando tutte le condizioni seguenti sono vere:

- [ ] Tutti gli 88 file originali sono stati ricollocati secondo il registro oppure hanno una destinazione alternativa documentata.
- [ ] Nessun file sorgente o test è andato perso.
- [ ] Tutti i moduli vengono risolti correttamente e tutti gli import sono aggiornati.
- [ ] Le interfacce pubbliche e `pub(crate)` necessarie sono mantenute.
- [ ] Tutti i test vengono ancora compilati ed eseguiti.
- [ ] Il comportamento dei sistemi non è stato modificato intenzionalmente.
- [ ] CI Linux, CI Windows e avvio headless risultano verdi sullo SHA finale.
- [ ] La roadmap principale riporta il completamento del passaggio preparatorio.

## 10. Stato attuale

**Piano creato; refactoring del codice non ancora iniziato.** Questo documento contiene la struttura proposta e il registro completo degli 88 percorsi sorgente. Nessun sorgente è stato spostato o modificato durante la preparazione del piano.
