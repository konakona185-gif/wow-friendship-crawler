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
10. Tutte le modifiche del refactoring devono essere effettuate e committate esclusivamente sul branch `refactor/src-domain-layout`: non modificare direttamente `main`.
11. `main` rimane il branch stabile di riferimento; il branch di refactoring viene integrato in `main` soltanto dopo audit, CI finale verde e revisione tramite Pull Request.
12. I file di codice e test devono restare, ove possibile, entro **1.000 righe fisiche**. Oltre 1.000 righe va valutata e pianificata la suddivisione per responsabilità; **1.500 righe è un limite eccezionale**, ammesso soltanto con motivazione documentata.

## 3. Baseline e inventario

- Repository: `konakona185-gif/wow-friendship-crawler`
- Branch di riferimento per l'inventario iniziale: `main`
- Branch di lavoro dedicato: `refactor/src-domain-layout`
- SHA iniziale del branch di lavoro: `454874a358a18e7e565043d797e3c42ddc46323f` (identico allo SHA di `main` al momento della creazione)
- File Rust censiti sotto `src/`: **88**
- Directory censite sotto `src/`: **18**
- Dimensione complessiva iniziale: **913.840 byte**
- Percorsi sorgente mappati: **88/88**
- Destinazioni duplicate nella proposta: **0**
- Nuovi moduli aggregatori di dominio previsti sotto `combat/`: **6** (`aura`, `core`, `effects`, `enemy`, `player`, `targeting`)
- La scansione GitHub dell'albero iniziale risultava completa (`truncated: false`).
- Audit iniziale delle dimensioni: **4 file su 88 superano 1.000 righe**; nessun file supera 1.500 righe. I file già oltre il target sono elencati e mappati per la suddivisione nella sezione 5.

La baseline di codice è la revisione `55e001ec7dc06f5d9bbb3107f3b5ef3f96354b38`, verificata dalla Run **#522**: 660 test superati su Linux e Windows e avvio headless superato. Il branch di lavoro è stato creato successivamente dallo SHA `454874a358a18e7e565043d797e3c42ddc46323f`, che in quel momento coincideva con `main`; i commit successivi su `main` riguardavano soltanto documentazione. Prima degli spostamenti controllare la CI disponibile e registrare lo SHA effettivo. Durante il refactoring il conteggio dei test va confrontato con il baseline: non devono andare persi test né introdursi regressioni.

## 4. Modalità di lavoro e gestione del documento

Questo file è il registro operativo. Per ogni gruppo di lavoro:

- segnare lo stato come `DA FARE`, `IN CORSO`, `COMPLETATO` o `BLOCCATO`;
- annotare i percorsi completati e le eventuali eccezioni;
- registrare gli eventuali cambi alla struttura proposta prima di applicarli;
- aggiornare il numero di file verificati e il risultato delle prove;
- non cancellare la cronologia delle decisioni: aggiungere una breve nota datata nella sezione finale.

Il piano può essere aggiornato quando l'ispezione del codice dimostra che una destinazione è inappropriata. In quel caso si aggiorna prima la tabella vecchio percorso → nuovo percorso e poi si sposta il file. Non lasciare mappature obsolete.

### Regola dimensionale dei file

La regola si applica a tutti i file Rust sotto `src/`, inclusi sorgenti, moduli `mod.rs` e file di test. Il conteggio è quello delle righe fisiche mostrate dall'editor, comprese righe vuote e commenti, così la soglia è verificabile in modo uniforme.

- **Obiettivo:** mantenere ogni file a **1.000 righe o meno**, quando la suddivisione in moduli coerenti è possibile.
- **Oltre 1.000 righe:** il file deve essere esaminato e, se contiene responsabilità, sezioni o gruppi di test separabili, suddiviso durante questa Fase 3.5. Non basta spostarlo in una nuova directory lasciandolo comunque sovradimensionato.
- **Oltre 1.500 righe:** non è accettabile come soluzione ordinaria. È ammesso soltanto in casi eccezionali, con motivazione tecnica esplicita registrata qui e approvazione dell'utente.
- Non creare frammenti arbitrari per rispettare un numero: ogni file risultante deve avere una responsabilità chiara, nomi descrittivi e interfacce modulari coerenti.
- La regola vale anche per i test. Fixture e helper condivisi possono rimanere in `tests/mod.rs`; i gruppi di test vanno separati in file distinti per area funzionale.

#### Audit iniziale dei file oltre il target

Conteggio eseguito su tutti gli 88 file `.rs` di `src/` presenti su `main` al checkpoint iniziale (7 novembre 2026), contando le righe fisiche del file e senza aggiungere una riga fittizia dopo il newline finale:

| File originale | Righe | Intervento previsto |
|---|---:|---|
| `src/combat/aura_runtime/tests.rs` | 1.303 | Separare test di stacking/applicazione, test periodici e test di durata/validazione/Pandemic. |
| `src/combat/effect_execution.rs` | 1.152 | Separare implementazione ed errori dai test inline; trasferire i test in un modulo dedicato. |
| `src/combat/effect_runtime/tests.rs` | 1.007 | Mantenere fixture condivise in `tests/mod.rs` e separare test della coda/risoluzione da test di esecuzione/eventi runtime. |
| `src/combat/summon_integration_tests.rs` | 1.137 | Mantenere fixture condivise in `tests/mod.rs` e separare test di spawn/fazioni, combattimento, ciclo di vita e contenuti Classic. |

File vicini alla soglia da monitorare durante gli interventi: `src/combat/damage/mitigation.rs` (893), `src/combat/effect_dispatcher/tests.rs` (914), `src/combat/enemy_ai/tests.rs` (906), `src/combat/events.rs` (952), `src/combat/effect_runtime/aura_tests.rs` (997) e `src/combat/summons.rs` (993). Sono sotto soglia, quindi non vanno frammentati automaticamente: vanno però riesaminati se le modifiche previste li portano oltre 1.000 righe.

### Regola dei branch

- `main` è il branch stabile e non va modificato direttamente durante questa attività.
- `refactor/src-domain-layout` è l'unico branch sul quale creare o modificare file, spostare sorgenti e registrare commit relativi al refactoring.
- Per i file non ancora modificati, `main` può essere consultato come baseline; dopo che un file è stato spostato o modificato, la versione di riferimento per il lavoro successivo è quella presente nel branch di refactoring.
- Non eseguire merge di `main` nel branch di lavoro automaticamente. Se emergesse una necessità concreta di sincronizzazione, valutarla prima e registrarla.
- L'integrazione in `main` avverrà soltanto tramite Pull Request, dopo il completamento delle checklist e una CI finale verde su Linux, Windows e headless.

## 5. Fasi operative

### Fase 0 – Preparazione e salvaguardia

**Stato: IN CORSO — branch dedicato creato e verificato; restano i controlli prima dei primi spostamenti.**

- [x] Verificare `main`, lo SHA iniziale e la baseline di codice.
- [x] Creare il branch dedicato `refactor/src-domain-layout` dallo SHA `454874a358a18e7e565043d797e3c42ddc46323f`.
- [ ] Verificare che il branch attivo sia `refactor/src-domain-layout` prima di ogni gruppo di modifiche.
- [ ] Verificare che il working tree sia pulito prima di iniziare ciascun gruppo.
- [ ] Conservare questo documento in `.github/workflows/`, insieme alla roadmap e al workflow YAML.
- [ ] Conservare l'inventario degli 88 file e usare la tabella della sezione 7 come checklist.
- [ ] Controllare che il branch contenga soltanto modifiche pertinenti al refactoring.
- [ ] Eseguire e registrare il controllo dimensionale dei file Rust, usando l'audit iniziale della sezione 4 come baseline.
- [ ] Pianificare la suddivisione di tutti i file oltre 1.000 righe prima di chiudere il refactoring.
- [ ] Effettuare tutti i commit di lavoro esclusivamente su `refactor/src-domain-layout`; non committare direttamente su `main`.
- [ ] A refactoring concluso, eseguire audit e CI finale, quindi preparare una Pull Request verso `main`. Non unire il branch prima della revisione e dell'esito CI verde.

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
- [ ] Suddividere `aura_runtime/tests.rs` (1.303 righe) in gruppi coerenti: `stacking_tests.rs`, `periodic_tests.rs` e `lifecycle_validation_tests.rs`, lasciando fixture condivise in `tests/mod.rs`.
- [ ] `core/`: eventi di combattimento, healing, health pool, risorse e statistiche.
- [ ] `damage/`: critici e mitigazione.
- [ ] `death/` e `displacement/`: relativi sistemi e test.
- [ ] `effects/`: custom effects, dispatcher, execution, magnitude, resolution, targeting e runtime.
- [ ] Suddividere `effect_execution.rs` (1.152 righe) in implementazione e modulo test dedicato.
- [ ] Suddividere `effect_runtime/tests.rs` (1.007 righe) in `queue_tests.rs` ed `execution_tests.rs`, con fixture condivise in `tests/mod.rs`.
- [ ] `enemy/`: AI e attacchi dei nemici.
- [ ] `player/`: attacchi del personaggio giocante.
- [ ] `summon/`: entità evocate, AI, attacchi, lifecycle e test di integrazione.
- [ ] `targeting/`: indicatore e selezione del bersaglio.
- [ ] Aggiornare `combat/mod.rs` e i re-export per mantenere stabile l'interfaccia esterna.

**Controlli particolarmente importanti per `summon/`:**

- [ ] Mantenere `SummonedCreature`, `AllySummon`, `AllySummonAi`, `SummonFaction` e gli altri tipi pubblici disponibili tramite `crate::combat` dove sono già utilizzati.
- [ ] Mantenere accessibile `SummonCombatProfile` con la visibilità `pub(crate)` prevista.
- [ ] Controllare gli import tra `summon/`, `enemy/` e `combat/core`; non assumere che `use super::*` continui a riferirsi agli stessi simboli dopo lo spostamento.
- [ ] Spostare i test di integrazione in `summon/tests/` e dividerli in `mod.rs` (fixture condivise), `spawn_tests.rs`, `combat_tests.rs`, `lifecycle_tests.rs` e `classic_pack_tests.rs`; aggiornare il percorso del modulo test in `combat/mod.rs`.
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
- [ ] Controllare la lunghezza di ogni file Rust dopo gli spostamenti: obiettivo massimo 1.000 righe; ogni file oltre tale soglia deve avere una suddivisione motivata o un'eccezione registrata; nessun file oltre 1.500 senza approvazione esplicita.

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
- I test inline nei file sorgente sotto la soglia possono restare dove sono se la struttura è coerente. L'eccezione già individuata è `effect_execution.rs`: poiché il file supera 1.000 righe sommando implementazione e test, il modulo di test viene estratto in `effects/execution/tests/mod.rs`.
- I file di test già separati ma oltre 1.000 righe vengono suddivisi per gruppi funzionali, mantenendo fixture condivise nel relativo `tests/mod.rs`.
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
│   │   │   │   ├── lifecycle_validation_tests.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── periodic_tests.rs
│   │   │   │   └── stacking_tests.rs
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
│   │   │   │   ├── execution_tests.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── periodic_tests.rs
│   │   │   │   └── queue_tests.rs
│   │   │   └── mod.rs
│   │   ├── custom.rs
│   │   ├── execution/
│   │   │   ├── mod.rs
│   │   │   └── tests/
│   │   │       └── mod.rs
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
│   │   │   ├── classic_pack_tests.rs
│   │   │   ├── combat_tests.rs
│   │   │   ├── lifecycle_tests.rs
│   │   │   ├── mod.rs
│   │   │   └── spawn_tests.rs
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
| `src/combat/aura_runtime/tests.rs` | `src/combat/aura/runtime/tests/mod.rs`, `src/combat/aura/runtime/tests/stacking_tests.rs`, `src/combat/aura/runtime/tests/periodic_tests.rs`, `src/combat/aura/runtime/tests/lifecycle_validation_tests.rs` |
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
| `src/combat/effect_execution.rs` | `src/combat/effects/execution/mod.rs` (implementazione), `src/combat/effects/execution/tests/mod.rs` (test inline estratti) |
| `src/combat/effect_magnitude.rs` | `src/combat/effects/magnitude.rs` |
| `src/combat/effect_resolution.rs` | `src/combat/effects/resolution.rs` |
| `src/combat/effect_runtime.rs` | `src/combat/effects/runtime/mod.rs` |
| `src/combat/effect_runtime/aura_tests.rs` | `src/combat/effects/runtime/tests/aura_tests.rs` |
| `src/combat/effect_runtime/corruption_tests.rs` | `src/combat/effects/runtime/tests/corruption_tests.rs` |
| `src/combat/effect_runtime/custom_effect_tests.rs` | `src/combat/effects/runtime/tests/custom_effect_tests.rs` |
| `src/combat/effect_runtime/displacement_tests.rs` | `src/combat/effects/runtime/tests/displacement_tests.rs` |
| `src/combat/effect_runtime/periodic_tests.rs` | `src/combat/effects/runtime/tests/periodic_tests.rs` |
| `src/combat/effect_runtime/tests.rs` | `src/combat/effects/runtime/tests/mod.rs`, `src/combat/effects/runtime/tests/queue_tests.rs`, `src/combat/effects/runtime/tests/execution_tests.rs` |
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
| `src/combat/summon_integration_tests.rs` | `src/combat/summon/tests/mod.rs`, `src/combat/summon/tests/spawn_tests.rs`, `src/combat/summon/tests/combat_tests.rs`, `src/combat/summon/tests/lifecycle_tests.rs`, `src/combat/summon/tests/classic_pack_tests.rs` |
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
| Fase 0 – Preparazione | IN CORSO | Branch `refactor/src-domain-layout` creato dallo SHA `454874a...`; controlli di pulizia e conferma branch attivo ancora da completare. |
| Fase 1 – Moduli top-level | DA FARE | |
| Fase 2 – Dominio combat | DA FARE | |
| Fase 3 – Altri domini | DA FARE | |
| Fase 4 – Audit moduli/import/test | DA FARE | |
| Fase 5 – Verifica e chiusura | DA FARE | |

### Registro delle decisioni e delle eccezioni

| Data | Decisione / modifica al piano | Motivazione | Stato |
|---|---|---|---|
| 2026-11-07 | Creato il piano iniziale con inventario completo di 88 file. | Preparare la riorganizzazione prima di iniziare la Fase 4 del progetto. | REGISTRATO |
| 2026-11-07 | Creato `refactor/src-domain-layout` dallo SHA `454874a358a18e7e565043d797e3c42ddc46323f`; `main` resta intatto. | Isolare tutti gli spostamenti e le modifiche strutturali dal branch stabile. | REGISTRATO |
| 2026-11-07 | Aggiunta la regola dimensionale: obiettivo massimo 1.000 righe per file, 1.500 soltanto in casi eccezionali; censiti i quattro file oltre 1.000 e definita la loro suddivisione. | La riorganizzazione deve anche ridurre i file monolitici, non limitarsi a spostarli. | REGISTRATO |

## 9. Criteri di completamento

Il refactoring è concluso soltanto quando tutte le condizioni seguenti sono vere:

- [ ] Tutti gli 88 file originali sono stati ricollocati secondo il registro oppure hanno una destinazione alternativa documentata.
- [ ] Nessun file sorgente o test è andato perso.
- [ ] Tutti i moduli vengono risolti correttamente e tutti gli import sono aggiornati.
- [ ] Le interfacce pubbliche e `pub(crate)` necessarie sono mantenute.
- [ ] Tutti i test vengono ancora compilati ed eseguiti.
- [ ] Nessun file supera 1.000 righe, salvo eccezioni motivate; nessun file supera 1.500 righe senza approvazione esplicita e motivazione documentata.
- [ ] Il comportamento dei sistemi non è stato modificato intenzionalmente.
- [ ] CI Linux, CI Windows e avvio headless risultano verdi sullo SHA finale.
- [ ] La roadmap principale riporta il completamento del passaggio preparatorio.

## 10. Stato attuale

**Piano creato; branch dedicato verificato; refactoring del codice non ancora iniziato.** Il lavoro operativo deve proseguire esclusivamente su `refactor/src-domain-layout`. `main` non è stato modificato dal refactoring. Questo documento contiene la struttura proposta e il registro completo degli 88 percorsi sorgente. Nessun sorgente è stato ancora spostato o modificato.
