# Refactoring – piano operativo permanente

> Documento di lavoro per la riorganizzazione strutturale del codice Rust sotto `src/`.
> Questo file va mantenuto nel repository come riferimento vivo: si aggiornano qui stato, decisioni, mappature e verifiche man mano che il refactoring procede.

Ultimo aggiornamento: 9 novembre 2026.

**Stato generale: Fase 3.5 completata tecnicamente.** I Gruppi 1–6 sono completati e sottoposti ad audit statico. Il Gruppo 7 è completato con GitHub Actions Run #544 interamente verde su Linux e Windows. L'integrazione in `main` deve ancora avvenire tramite Pull Request.

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
9. Non iniziare la Fase 4 del progetto finché questo refactoring non è completato, verificato e integrato in `main`.
10. Tutte le modifiche del refactoring devono essere effettuate e committate esclusivamente sul branch `refactor/src-domain-layout`: non modificare direttamente `main`.
11. `main` rimane il branch stabile di riferimento; il branch di refactoring viene integrato in `main` soltanto dopo audit, CI finale verde e revisione tramite Pull Request.
12. Valutare la modularizzazione di ogni file di codice e test in base alle responsabilità, indipendentemente dalla dimensione. La soglia di **1.000 righe fisiche** è un limite di attenzione che impone una valutazione, non la dimensione minima per suddividere un file. Quando responsabilità distinte possono essere separate in unità coerenti con un vantaggio architetturale concreto, suddividere anche sotto tale soglia. Applicare il criterio a tutti i domini e ai test; raggruppare i test per funzionalità e comportamento, mantenere fixture/helper condivisi in moduli comuni ed evitare duplicazioni. Non frammentare artificialmente in file minuscoli o privi di una responsabilità chiara. **1.500 righe è un limite eccezionale**, ammesso soltanto con motivazione documentata e approvazione dell'utente.

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
- Audit iniziale delle dimensioni: **4 file su 88 superano la soglia di attenzione di 1.000 righe**; nessun file supera 1.500 righe. I file sono elencati nella sezione 5 e vengono esaminati per responsabilità separabili; la loro suddivisione è prevista dove offre un confine coerente.

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

- **Criterio principale:** valutare ogni file, a qualunque dimensione, per responsabilità separabili. Se una suddivisione naturale migliora i confini architetturali, la leggibilità o l'organizzazione dei test senza introdurre frammentazione artificiale, effettuare la suddivisione anche sotto le 1.000 righe.
- **Soglia di attenzione:** 1.000 righe fisiche non è il criterio minimo per suddividere. Al suo superamento, esaminare obbligatoriamente il file e suddividerlo quando responsabilità, sezioni o gruppi di test possono essere separati in unità coerenti; se non si suddivide, registrare la motivazione quando il file resta oltre la soglia.
- **Oltre 1.500 righe:** non è accettabile come soluzione ordinaria. È ammesso soltanto in casi eccezionali, con motivazione tecnica esplicita registrata qui e approvazione dell'utente.
- Non creare frammenti arbitrari per rispettare un numero: ogni file risultante deve avere una responsabilità chiara, nomi descrittivi e interfacce modulari coerenti.
- La regola vale in tutti i domini per sorgenti, moduli, test unitari e test d'integrazione. Raggruppare i test simili per funzionalità e comportamento, separando responsabilità differenti anche quando i file sono brevi. Mantenere fixture e helper condivisi in un modulo comune (per esempio `tests/mod.rs`) e riutilizzarli senza duplicare codice. Evitare file minuscoli quando non c'è un vantaggio architetturale concreto.

#### Audit iniziale dei file oltre il target

Conteggio eseguito su tutti gli 88 file `.rs` di `src/` presenti su `main` al checkpoint iniziale (7 novembre 2026), contando le righe fisiche del file e senza aggiungere una riga fittizia dopo il newline finale:

| File originale | Righe | Intervento previsto |
|---|---:|---|
| `src/combat/aura_runtime/tests.rs` | 1.303 | Separare test di stacking/applicazione, test periodici e test di durata/validazione/Pandemic. |
| `src/combat/effect_execution.rs` | 1.152 | Separare implementazione ed errori dai test inline; trasferire i test in un modulo dedicato. |
| `src/combat/effect_runtime/tests.rs` | 1.007 | Mantenere fixture condivise in `tests/mod.rs` e separare test della coda/risoluzione da test di esecuzione/eventi runtime. |
| `src/combat/summon_integration_tests.rs` | 1.137 | Mantenere fixture condivise in `tests/mod.rs` e separare test di spawn/fazioni, combattimento, ciclo di vita e contenuti Classic. |

Altri file erano vicini alla soglia al checkpoint iniziale: `src/combat/damage/mitigation.rs` (893), `src/combat/effect_dispatcher/tests.rs` (914), `src/combat/enemy_ai/tests.rs` (906), `src/combat/events.rs` (952), `src/combat/effect_runtime/aura_tests.rs` (997) e `src/combat/summons.rs` (993). Questi conteggi sono una baseline storica: essere sotto 1.000 righe non esclude una suddivisione. Durante i gruppi rimanenti e l'audit complessivo vanno esaminati per responsabilità come tutti gli altri file; non vanno però frammentati per il solo conteggio.

### Regola dei branch

- `main` è il branch stabile e non va modificato direttamente durante questa attività.
- `refactor/src-domain-layout` è l'unico branch sul quale creare o modificare file, spostare sorgenti e registrare commit relativi al refactoring.
- Per i file non ancora modificati, `main` può essere consultato come baseline; dopo che un file è stato spostato o modificato, la versione di riferimento per il lavoro successivo è quella presente nel branch di refactoring.
- Non eseguire merge di `main` nel branch di lavoro automaticamente. Se emergesse una necessità concreta di sincronizzazione, valutarla prima e registrarla.
- L'integrazione in `main` avverrà soltanto tramite Pull Request, dopo il completamento delle checklist e una CI finale verde su Linux, Windows e headless.

## 5. Fasi operative

### Fase 0 – Preparazione e salvaguardia

**Stato: PREPARAZIONE INIZIALE COMPLETATA E VERIFICATA (Gruppo 1). Le salvaguardie di branch e working tree restano obbligatorie per ogni gruppo; la verifica tecnica finale è stata completata nel Gruppo 7.**

- [x] Verificare `main`, lo SHA iniziale e la baseline di codice.
- [x] Creare il branch dedicato `refactor/src-domain-layout` dallo SHA `454874a358a18e7e565043d797e3c42ddc46323f`.
- [x] Verificare che il branch attivo sia `refactor/src-domain-layout` prima di ogni gruppo di modifiche; registrare branch e SHA nei checkpoint.
- [x] Verificare working tree e staging prima di ciascun gruppo, preservando le modifiche intenzionali già presenti. Il working tree non deve essere pulito quando contiene modifiche autorizzate dei gruppi precedenti.
- [x] Conservare questo documento in `.github/workflows/`, insieme alla roadmap e al workflow YAML.
- [x] Conservare l'inventario degli 88 file e usare la tabella della sezione 7 come checklist.
- [x] Controllare che le modifiche presenti sul branch siano pertinenti al refactoring.
- [x] Eseguire e registrare il controllo dimensionale dei file Rust, usando l'audit iniziale della sezione 4 come baseline.
- [x] Esaminare tutti i file oltre 1.000 righe e valutare la suddivisione quando le responsabilità sono separabili; documentare le scelte strutturali.
- [x] Esaminare moduli e test di tutti i domini, indipendentemente dalle dimensioni, applicando il criterio di responsabilità senza frammentazione artificiale.
- [x] Effettuare i commit di lavoro esclusivamente su `refactor/src-domain-layout`; nessun commit diretto sul branch `main`.
- [ ] Completare la preparazione e la revisione della Pull Request verso `main`; non unire il branch prima della revisione e dell'approvazione esplicita.

### Fase 1 – Conversione dei moduli di primo livello

**Stato: STRUTTURA COMPLETATA — audit statico superato; formattazione, compilazione e test verificati nella CI finale del Gruppo 7.**

Convertire i file che rappresentano moduli principali in directory con `mod.rs`, senza cambiare i nomi logici dei moduli importati da `main.rs`:

- [x] `camera.rs` → `camera/mod.rs`
- [x] `character.rs` → `character/mod.rs`
- [x] `combat.rs` → `combat/mod.rs`
- [x] `content.rs` → `content/mod.rs`
- [x] `fellowship.rs` → `fellowship/mod.rs`
- [x] `interaction.rs` → `interaction/mod.rs`
- [x] `navigation.rs` → `navigation/mod.rs`
- [x] `player.rs` → `player/mod.rs`
- [x] Integrare `enemy_navigation.rs` come sottosistema `navigation/enemy/mod.rs`.

Controlli specifici:

- [x] Aggiornare `main.rs` se cambia il punto di accesso al plugin di navigazione nemica; idealmente `main.rs` importa `EnemyNavigationPlugin` da `navigation` e non mantiene un modulo top-level separato `enemy_navigation`.
- [x] Assicurarsi che `mod.rs` continui a esporre gli stessi simboli pubblici richiesti da `main.rs` e dagli altri domini.
- [x] Verificare che la conversione `file.rs` → `directory/mod.rs` non abbia cambiato la risoluzione dei moduli figli.

### Fase 2 – Riordino del dominio combat

**Stato: STRUTTURA COMPLETATA — audit statico superato; verifiche tecniche superate nella CI finale del Gruppo 7.**

Spostare i sistemi di combattimento nelle aree proposte nella sezione 6 e nell'albero della sezione 7.

- [x] `aura/`: esecuzione, runtime, sistemi e refresh delle aure.
- [x] Suddividere `aura_runtime/tests.rs` (1.303 righe) in gruppi coerenti: `stacking_tests.rs`, `periodic_tests.rs` e `lifecycle_validation_tests.rs`, lasciando fixture condivise in `tests/mod.rs`.
- [x] `core/`: eventi di combattimento, healing, health pool, risorse e statistiche.
- [x] `damage/`: critici e mitigazione.
- [x] `death/` e `displacement/`: relativi sistemi e test.
- [x] `effects/`: custom effects, dispatcher, execution, magnitude, resolution, targeting e runtime.
- [x] Suddividere `effect_execution.rs` (1.152 righe) in implementazione e modulo test dedicato.
- [x] Suddividere `effect_runtime/tests.rs` (1.007 righe) in `queue_tests.rs` ed `execution_tests.rs`, con fixture condivise in `tests/mod.rs`.
- [x] `enemy/`: AI e attacchi dei nemici.
- [x] `player/`: attacchi del personaggio giocante.
- [x] `summon/`: entità evocate, AI, attacchi, lifecycle e test di integrazione.
- [x] `targeting/`: indicatore e selezione del bersaglio.
- [x] Aggiornare `combat/mod.rs` e i re-export per mantenere stabile l'interfaccia esterna (confronto statico dei 67 re-export).

**Controlli particolarmente importanti per `summon/`:**

- [x] Mantenere `SummonedCreature`, `AllySummon`, `AllySummonAi`, `SummonFaction` e gli altri tipi pubblici disponibili tramite `crate::combat` dove sono già utilizzati.
- [x] Mantenere accessibile `SummonCombatProfile` con la visibilità `pub(crate)` prevista.
- [x] Controllare gli import tra `summon/`, `enemy/` e `combat/core`; non assumere che `use super::*` continui a riferirsi agli stessi simboli dopo lo spostamento.
- [x] Spostare i test di integrazione in `summon/tests/` e dividerli in `mod.rs` (fixture condivise), `spawn_tests.rs`, `combat_tests.rs`, `lifecycle_tests.rs` e `classic_pack_tests.rs`; aggiornare il percorso del modulo test in `combat/mod.rs`.
- [x] Confrontare staticamente i sorgenti per confermare che le regole di fazione delle summon e la distinzione tra `Character`, `Enemy` e `AllySummon` non siano state modificate.

### Fase 3 – Riordino di Content, Character, Player, Navigation, Fellowship e Interaction

**Stato: COMPLETATA STRUTTURALMENTE E VERIFICATA STATICAMENTE — verifiche tecniche Rust completate nella CI finale tramite GitHub Actions.**

- [x] Spostare `content.rs` e `content/database.rs` nelle rispettive directory `mod.rs`, mantenendo i modelli e le API esistenti.
- [x] Riordinare i test del database in `content/database/tests/` senza perdere i test di aura, classic pack, reference e registration.
- [x] Verificare `character/` e i relativi test secondo responsabilità e coesione.
- [x] Verificare `player/` mantenendo input, movement e movement helpers; condividere gli helper di movimento duplicati con Navigation.
- [x] Accorpare la navigazione nemica sotto `navigation/enemy/` e mantenere il plugin registrato una sola volta.
- [x] Controllare gli import cross-domain in `main.rs`, `combat`, `player`, `navigation` e `interaction`.

### Gruppo 4 – Suddivisione per responsabilità di moduli e test

**Stato: COMPLETATO STRUTTURALMENTE E VERIFICATO STATICAMENTE — formattazione, compilazione e test superati nella CI finale tramite GitHub Actions.**

Questo gruppo applica il criterio di modularizzazione a qualunque dimensione. Si limita a separare responsabilità già presenti nei moduli e nei test, senza avviare il riordino dei domini previsto dal Gruppo 5.

- [x] `camera/`: separare test di controlli/distanza, transizioni di follow e visibilità/fade (15 test mantenuti in tre moduli).
- [x] `combat/core/events/`: estrarre i 33 test di validazione contesto, coda eventi e sottoscrizioni; mantenere fixture comuni.
- [x] `combat/summon/`: estrarre e separare 18 test inline di tipi/lifecycle, spawn e runtime effetti; mantenere fixture comuni.
- [x] `combat/effects/runtime/tests/`: separare i 10 test aura di operazioni, lifecycle/periodicità e dispatch reattivo; helper reattivi condivisi nel parent.
- [x] `combat/effects/dispatcher/tests/`: separare i 19 test di selezione subscriber e runtime reattivo; conservare i 5 test in `area_tests`.
- [x] `combat/enemy/ai/tests/`: separare 22 test di acquisizione/selezione bersagli da aggro, salute e lifecycle del leash.
- [x] `combat/enemy/attacks/tests/`: separare 17 test di idoneità del bersaglio da risoluzione danno/cooldown.
- [x] `combat/player/attacks/tests/`: separare 15 test di idoneità al cast da risoluzione di danno, abilità e threat.
- [x] `content/loader/`: separare 10 test di serializzazione/compatibilità, registrazione e validazione/transazioni.
- [x] Confermare staticamente che i 159 nomi di test dell'inventario iniziale siano conservati una sola volta; fixture condivise centralizzate e dipendenze dei moduli/percorsi verificate.
- [x] Eseguire `git diff --check`; formattazione, compilazione e test sono stati verificati nella CI finale tramite GitHub Actions.

### Gruppo 5 – Riordino degli altri domini

**Stato: COMPLETATO STRUTTURALMENTE E VERIFICATO STATICAMENTE — formattazione, compilazione e test superati nella CI finale tramite GitHub Actions.**

Ambito: `content/`, `character/`, `player/`, `navigation/`, `fellowship/` e `interaction/`. Applicare i confini già indicati nella mappa completa della sezione 7 e valutare anche responsabilità distinte sotto le 1.000 righe. Mantenere API, visibilità, re-export, comportamento, test e fixture. Gli aggregatori top-level già convertiti nel Gruppo 2 restano parte della struttura di destinazione.

Nota di pianificazione emersa dall'ispezione iniziale: la mappa assegna `content/database.rs` a `content/database/mod.rs` e `content/database/tests.rs` a `content/database/tests/mod.rs`, ma i due wrapper risultano ancora nelle posizioni legacy. Il parent dei test contiene fixture condivise e quattro test inline, tre dei quali costituiscono un gruppo coerente di validazione dei talenti; la mappa include quindi `talent_validation_tests.rs` per separare quel gruppo mantenendo le fixture comuni e il test singolo sulle dipendenze dei pacchetti nel parent. La mappa è aggiornata prima degli spostamenti.

L'audit comparativo ha inoltre rilevato che gli helper di movimento di Player e Navigation duplicano gli stessi algoritmi di collisione, discesa, gravità e step-up con costanti e nomi specifici per dominio. Poiché valori e corpi sono equivalenti, il piano introduce `src/movement/mod.rs` come modulo interno condiviso; i wrapper Player/Enemy conserveranno i nomi locali, i valori e i test esistenti tramite re-export, mentre la registrazione del modulo sarà privata in `main.rs`. Questo collegamento cross-domain è necessario per eliminare la duplicazione senza cambiare le formule.

Esito del gruppo: i wrapper Content sono stati convertiti e i tre test di validazione talenti separati; gli helper Player/Enemy ora delegano al modulo interno condiviso mantenendo alias, costante `JUMP_SPEED` specifica del player e test inline. Character, Player, Navigation, Fellowship e Interaction sono stati valutati secondo il criterio aggiornato; dove le responsabilità erano già coese non è stata introdotta frammentazione. L'inventario statico dei test dei sei domini è invariato (149 occorrenze, 146 nomi distinti). La risoluzione dei moduli, i percorsi dei file, i riferimenti alle fixture e i macro di percorso sono stati controllati; nessun `#[path]`, `include_str!` o `include_bytes!` è presente nei domini coinvolti, salvo due `include_str!` delle fixture Classic già esistenti e con percorsi ancora corretti. `git diff --check` è superato. Nessun controllo Cargo, formattazione, compilazione o test è stato eseguito durante il singolo gruppo; tali controlli sono stati completati nella CI finale.

- [x] Convertire i wrapper `content/database.rs` e `content/database/tests.rs` in `database/mod.rs` e `database/tests/mod.rs`; isolare i tre test di validazione dei talenti in `talent_validation_tests.rs`, mantenendo le fixture condivise.
- [x] Auditare il resto di Content (inclusi loader e definizioni per tipo) senza modificare le suddivisioni completate nel Gruppo 4 salvo necessità documentata.
- [x] Auditare Character e i relativi test per responsabilità e fixture condivise; i quattro test formano una suite coerente sulla matrice razza-classe e le fazioni.
- [x] Auditare Player (input, movement e helper) e i collegamenti con combat/navigation.
- [x] Auditare Navigation, mantenendo il plugin registrato una sola volta e i moduli nemici sotto `navigation/enemy/`.
- [x] Centralizzare gli helper di movimento identici di Player/Enemy in `movement/mod.rs`, mantenendo gli alias interni specifici e tutte le suite di test esistenti.
- [x] Valutare Fellowship e Interaction: i rispettivi moduli rappresentano un unico sottosistema membership e un unico flusso pointer hover/click; non sono stati frammentati.
- [x] Verificare `mod`, import, re-export, visibilità, `use super::*`, `#[path]`, `include_str!`, `include_bytes!`, percorsi e test dei domini interessati.
- [x] Confrontare il multinsieme dei nomi dei test prima/dopo (149 occorrenze, 146 nomi distinti); nessuna differenza e fixture condivise mantenute.
- [x] Eseguire `git diff --check`; formattazione, compilazione e test sono stati verificati nella CI finale tramite GitHub Actions.

### Gruppo 6 – Audit completo di moduli, import e test

**Stato: COMPLETATO STATICAMENTE — Gruppo 6 concluso sul branch `refactor/src-domain-layout`; verifica tecnica successivamente completata nel Gruppo 7.**

Rilievi iniziali del Gruppo 6: la tabella delle destinazioni non riportava i sei aggregatori combat generati durante il refactoring e registrava due volte il modulo movement condiviso; il checkout iniziale conteneva 133 file Rust. La mappa è stata corretta prima di ulteriori spostamenti. La suite di `targeting/selection` conteneva due gruppi distinti per interazione (selezione/ciclo tramite tastiera e selezione tramite click), con fixture comuni; la mappa è stata aggiornata e i due gruppi sono ora separati, mantenendo le fixture nel parent.

- [x] Cercare ogni dichiarazione `mod` e verificare che punti al file effettivo: 0 destinazioni mancanti nel controllo statico.
- [x] Cercare ogni attributo `#[path = ...]` e aggiornare i percorsi spostati: nessun attributo `#[path]` presente in `src/`.
- [x] Controllare ogni `use super::*`: i moduli parent e le fixture condivise sono stati verificati staticamente; la risoluzione completa dei simboli è stata successivamente confermata in compilazione.
- [x] Controllare i re-export `pub use` e `pub(crate) use`; non allargare la visibilità senza motivo: inventario dei 67 re-export combat e 49 Content invariato rispetto a HEAD.
- [x] Controllare `include_str!`, `include_bytes!`, path relativi e riferimenti a file non Rust: tutti i target statici risolvono; nessun `include_bytes!`.
- [x] Verificare che i test esterni e quelli inline siano tutti ancora inclusi con `#[cfg(test)]` e dichiarazioni `mod` esistenti.
- [x] Cercare riferimenti testuali ai vecchi percorsi sotto `src/`: nessun riferimento ai vecchi moduli combat o `enemy_navigation`.
- [x] Confrontare numero e multinsieme dei test con HEAD: 660 occorrenze e 656 nomi distinti in entrambi, nessuna differenza.
- [x] Controllare la lunghezza fisica di ogni file Rust: 135 file, massimo 893 righe (`combat/damage/mitigation.rs`), nessun file oltre 1.000 o 1.500 righe.
- [x] Valutare responsabilità e gruppi di test di tutti i domini a qualunque dimensione; fixture/helper condivisi e assenza di frammentazioni prive di vantaggio verificati. Separati i test tab/click di targeting.

Esito del Gruppo 6 – audit statico complessivo:

- Mappatura: 88/88 percorsi sorgente del baseline sono presenti nella tabella e producono 129 destinazioni operative; sono inoltre registrati 6 aggregatori combat generati. Il totale è 135 file Rust, corrispondenti ai 135 file `.rs` correnti: nessuna destinazione mancante o non esistente.
- Moduli e import: tutte le dichiarazioni `mod` esaminate puntano a un file o `mod.rs` esistente (0 errori statici). Sono stati controllati i `use super::*`, i confini parent e gli import cross-domain. La risoluzione dei nomi e delle visibilità è stata successivamente verificata dalla compilazione del Gruppo 7.
- Percorsi: nessun `#[path]`; tutti i `include_str!` risolvono a file presenti e non è presente `include_bytes!`. Nessun riferimento ai vecchi percorsi dei moduli riorganizzati è rimasto sotto `src/`.
- Re-export: confrontati con HEAD i 67 elementi re-esportati da `combat` e i 49 da `content`; nessuna differenza. `navigation` mantiene il re-export richiesto di `EnemyNavigationPlugin`; non sono state introdotte modifiche di visibilità nel Gruppo 6.
- Test: inventario da HEAD e checkout corrente pari a 660 funzioni annotate `#[test]`, 656 nomi distinti e multinsieme identico. La ripartizione registrata nei Gruppi 3–5 è coerente; la suite di targeting conserva tutti i 13 casi (7 tab/ciclo, 6 click), ora in due moduli e con fixture comuni nel parent.
- Modularizzazione: le suite camera, combat, content, character, player, navigation, fellowship e interaction risultano organizzate per dominio/responsabilità secondo i gruppi precedenti. L'audit ha aggiunto soltanto la suddivisione tab/click; gli altri file esaminati sotto 1.000 righe sono rimasti coesi o rappresentano moduli di integrazione/composizione per cui ulteriori file sarebbero frammentazione artificiale. Il file più lungo è `combat/damage/mitigation.rs` a 893 righe; i test restano una suite unitaria di mitigazione e le responsabilità armor/resistance/scuole fanno parte dello stesso dominio.
- Integrità e ordine: il diff di questo gruppo sposta soltanto test e aggiorna dichiarazioni/moduli e documentazione. Le formule e i sistemi di produzione non sono stati modificati. L'ordine dei 14 sistemi in `CombatPlugin` coincide con quello di HEAD; l'ordine dei plugin in `main.rs` è invariato.
- Limite della verifica: il controllo statico non sostituisce la compilazione. La risoluzione dei nomi e delle visibilità, la compilazione e i test sono stati successivamente verificati nella CI del Gruppo 7.
- `git diff --check`: superato; restano solo avvisi Git di conversione LF/CRLF su file preesistenti.

### Gruppo 7 – Verifica tecnica e chiusura

**Stato: COMPLETATO — GitHub Actions Run #544 interamente verde su Linux e Windows, inclusi compilazione, test, avvio headless e artifact.**

#### Configurazione workflow

File: `.github/workflows/build.yml`.

Il controllo di formattazione è bloccante: `cargo fmt --all -- --check` è eseguito come passo diretto, quindi un errore fallisce il job.

La matrice Linux/Windows e i controlli previsti sono stati mantenuti:
- `cargo fmt --all -- --check`;
- `cargo check --all-targets`;
- `cargo build --release`;
- `cargo test --release`;
- `cargo run --release` con `CI=true`;
- caricamento degli artifact Linux e Windows.

Trigger:
- push soltanto su `main`;
- Pull Request verso `main`;
- avvio manuale tramite `workflow_dispatch`.

#### Run #536 – Prima esecuzione

- Run ID: `42504899025`.
- Branch: `refactor/src-domain-layout`.
- SHA testato: `0cfcf2a61fff40500c7f9aac7ebec7c50cae7c77`.
- Evento: `workflow_dispatch`.
- Esito: **failure**.

Il job preliminare Rust è fallito durante `cargo fmt --all -- --check`.

Errore:
`expected item after attributes`

Percorso:
`src/combat/effects/dispatcher/tests/reactive_runtime_tests.rs`, riga 488.

La causa era una direttiva `#[cfg(test)]` rimasta orfana dopo lo spostamento della dichiarazione `mod area_tests` nel modulo parent `tests/mod.rs`, già protetto da `#[cfg(test)]`.

I job di compilazione e test Linux e Windows sono stati saltati perché dipendenti dal controllo preliminare. Non sono stati verificati in questa run:
- `cargo check`;
- build release;
- test release;
- avvio headless;
- artifact.

Run: https://github.com/konakona185-gif/wow-friendship-crawler/actions/runs/42504899025

#### Correzione pubblicata dopo la Run #536

- Rimossa esclusivamente la direttiva `#[cfg(test)]` orfana da `src/combat/effects/dispatcher/tests/reactive_runtime_tests.rs`.
- La dichiarazione `mod area_tests` rimane nel modulo parent `tests/mod.rs`, già protetto da `#[cfg(test)]`.
- Gli 11 test del file e le suite sibling sono stati preservati.
- Commit: `b69aefcf7efe844d8c9bf32227bf04ec6cfca2ba`.
- Messaggio: `Remove unnecessary lines from reactive_runtime_tests.rs`.

La correzione era necessaria ma non rappresentava una verifica della compilazione: i controlli tecnici sono stati completati dalla successiva CI.

#### Correzioni di compilazione e visibilità

Durante il processo di correzione degli errori di compilazione sono emerse due diagnostiche relative a `summon_lifecycle_system`:

- E0364: re-export non consentito per una funzione con visibilità insufficiente.
- E0603: funzione non accessibile dal modulo chiamante.

La causa era la visibilità `pub(super)` della funzione in `src/combat/summon/lifecycle.rs`.

La correzione ha portato la dichiarazione a:

`pub(in crate::combat)`

La funzione rimane circoscritta al dominio Combat, senza diventare pubblica all'intero crate.

Sono state inoltre corrette le visibilità dei sistemi e gli import necessari nei moduli e nei test interessati. La compilazione finale ha verificato che i problemi individuati fossero risolti.

#### Run #544 – Verifica tecnica finale

- Run ID: `42587661206`.
- Branch: `refactor/src-domain-layout`.
- SHA testato: `1a3c5f4d24fd25d599596b1d16a7fc4006d0106b`.
- Evento: `workflow_dispatch`.
- Esito: **success**.
- Run: https://github.com/konakona185-gif/wow-friendship-crawler/actions/runs/42587661206

Controllo preliminare Rust:
- Formattazione Rust: superata.
- Installazione dipendenze Linux: superata.
- Controllo statico di tutti i target Rust (`cargo check --all-targets`): superato.

Linux:
- Compilazione: superata.
- Test unitari: superati.
- Test headless: superati.
- Artifact `gioco-linux`: caricato correttamente.

Windows:
- Compilazione: superata.
- Test unitari: superati.
- Test headless: superati.
- Artifact `gioco-windows`: caricato correttamente.

Artifact verificati:
- `gioco-linux`, ID `13661140684`.
- `gioco-windows`, ID `13660802759`.

La Run #544 è il checkpoint conclusivo della Fase 3.5. I controlli configurati dal workflow sono stati completati con esito positivo sullo SHA esatto indicato. Non attribuire alla run un numero di test non riportato dai log.

## 6. Decisioni architetturali della struttura proposta

- `src/main.rs` resta il punto d'ingresso e non viene spostato.
- Ogni dominio top-level ha una directory con `mod.rs`.
- `combat/` viene diviso per responsabilità; non si crea un unico contenitore generico con file eterogenei.
- I sistemi evocati vengono raggruppati in `combat/summon/`; il relativo test di integrazione sta nello stesso dominio.
- I test già separati vengono raccolti in directory `tests/` del sistema proprietario.
- I test inline possono restare nel file sorgente quando condividono la stessa responsabilità e la struttura è coerente; possono essere estratti anche sotto la soglia quando formano un gruppo funzionale distinto e l'estrazione migliora concretamente l'organizzazione. L'eccezione già individuata è `effect_execution.rs`: i test inline sono estratti in `effects/execution/tests/mod.rs`.
- I file di test vengono raggruppati per funzionalità e comportamento, non soltanto in base alla dimensione. Le fixture e gli helper comuni restano nel relativo `tests/mod.rs` o altro modulo condiviso, senza duplicazioni.
- I file oltre 1.000 righe sono sempre esaminati; la suddivisione dipende dalla presenza di responsabilità coerenti e separabili, non dal solo conteggio. Lo stesso esame architetturale vale per file più brevi in tutti i domini.
- I file `.ron`, `Cargo.toml` e workflow CI non vengono spostati da questa attività.
- Le directory `combat/aura`, `combat/core`, `combat/effects`, `combat/enemy`, `combat/player` e `combat/targeting` avranno un `mod.rs` aggregatore ciascuna.

## 7. Struttura di destinazione e registro completo dei file

L'albero seguente rappresenta la destinazione pianificata. La tabella immediatamente successiva è la checklist autorevole: ogni file sorgente deve comparire una volta, e una sola, nella colonna delle destinazioni. La mappatura produce **129 destinazioni operative**; a queste si aggiungono **6 aggregatori combat generati**, non provenienti dall'inventario iniziale. Il totale previsto e verificato è quindi **135 file Rust**.

## Albero proposto

```text
src/
├── camera/
│   ├── tests/
│   │   ├── controls_tests.rs
│   │   ├── follow_tests.rs
│   │   ├── mod.rs
│   │   └── visibility_tests.rs
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
│   │   ├── events/
│   │   │   ├── tests/
│   │   │   │   ├── context_validation_tests.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── queue_tests.rs
│   │   │   │   └── subscription_tests.rs
│   │   │   └── mod.rs
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
│   │   │   │   ├── reactive_runtime_tests.rs
│   │   │   │   ├── subscriber_tests.rs
│   │   │   │   └── mod.rs
│   │   │   └── mod.rs
│   │   ├── runtime/
│   │   │   ├── tests/
│   │   │   │   ├── aura_tests/
│   │   │   │   │   ├── lifecycle_tests.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── operation_tests.rs
│   │   │   │   │   └── reactive_tests.rs
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
│   │   │   │   ├── aggro_lifecycle_tests.rs
│   │   │   │   ├── mod.rs
│   │   │   │   └── target_acquisition_tests.rs
│   │   │   └── mod.rs
│   │   ├── attacks/
│   │   │   ├── tests/
│   │   │   │   ├── damage_resolution_tests.rs
│   │   │   │   ├── eligibility_tests.rs
│   │   │   │   └── mod.rs
│   │   │   └── mod.rs
│   │   └── mod.rs
│   ├── player/
│   │   ├── attacks/
│   │   │   ├── tests/
│   │   │   │   ├── ability_resolution_tests.rs
│   │   │   │   ├── eligibility_tests.rs
│   │   │   │   └── mod.rs
│   │   │   └── mod.rs
│   │   └── mod.rs
│   ├── summon/
│   │   ├── unit_tests/
│   │   │   ├── mod.rs
│   │   │   ├── runtime_tests.rs
│   │   │   ├── spawn_tests.rs
│   │   │   └── types_tests.rs
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
│   │   │   │   ├── click_tests.rs
│   │   │   │   ├── mod.rs
│   │   │   │   └── tab_tests.rs
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
│   │   │   ├── registration_tests.rs
│   │   │   └── talent_validation_tests.rs
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
│   ├── loader/
│   │   └── tests/
│   │       ├── mod.rs
│   │       ├── registration_tests.rs
│   │       ├── serialization_tests.rs
│   │       └── validation_tests.rs
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
├── movement/
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
