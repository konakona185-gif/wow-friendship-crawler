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

**Stato: PREPARAZIONE INIZIALE COMPLETATA E VERIFICATA (Gruppo 1). Le salvaguardie di branch e working tree restano obbligatorie per ogni gruppo; la verifica tecnica finale appartiene al Gruppo 7.**

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
- [ ] Effettuare tutti i commit di lavoro esclusivamente su `refactor/src-domain-layout`; non committare direttamente su `main`.
- [ ] A refactoring concluso, eseguire audit e CI finale, quindi preparare una Pull Request verso `main`. Non unire il branch prima della revisione e dell'esito CI verde.

### Fase 1 – Conversione dei moduli di primo livello

**Stato: STRUTTURA COMPLETATA — audit statico superato; formattazione, compilazione e test rinviati alla verifica finale tramite GitHub Actions.**

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
- [x] Assicurarsi che `mod.rs` continui a esporre gli stessi simboli pubblici richiesti da `main.rs` e dagli altri domini (controllo statico).
- [x] Verificare che la conversione `file.rs` → `directory/mod.rs` non abbia cambiato la risoluzione dei moduli figli (controllo statico).

### Fase 2 – Riordino del dominio combat

**Stato: STRUTTURA COMPLETATA — audit statico superato; verifiche tecniche rinviate alla Fase 5.**

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

**Stato: COMPLETATA STRUTTURALMENTE E VERIFICATA STATICAMENTE — le verifiche tecniche Rust restano rinviate alla verifica finale tramite GitHub Actions.**

- [x] Spostare `content.rs` e `content/database.rs` nelle rispettive directory `mod.rs`, mantenendo i modelli e le API esistenti.
- [x] Riordinare i test del database in `content/database/tests/` senza perdere i test di aura, classic pack, reference e registration.
- [x] Verificare `character/` e i relativi test secondo responsabilità e coesione.
- [x] Verificare `player/` mantenendo input, movement e movement helpers; condividere gli helper di movimento duplicati con Navigation.
- [x] Accorpare la navigazione nemica sotto `navigation/enemy/` e mantenere il plugin registrato una sola volta.
- [x] Controllare gli import cross-domain in `main.rs`, `combat`, `player`, `navigation` e `interaction`.

### Gruppo 4 – Suddivisione per responsabilità di moduli e test

**Stato: COMPLETATO STRUTTURALMENTE E VERIFICATO STATICAMENTE — formattazione, compilazione e test sono rinviati alla verifica finale tramite GitHub Actions.**

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
- [x] Eseguire `git diff --check`; formattazione, compilazione e test sono rinviati alla verifica finale tramite GitHub Actions.

### Gruppo 5 – Riordino degli altri domini

**Stato: COMPLETATO STRUTTURALMENTE E VERIFICATO STATICAMENTE — formattazione, compilazione e test sono rinviati alla verifica finale tramite GitHub Actions.**

Ambito: `content/`, `character/`, `player/`, `navigation/`, `fellowship/` e `interaction/`. Applicare i confini già indicati nella mappa completa della sezione 7 e valutare anche responsabilità distinte sotto le 1.000 righe. Mantenere API, visibilità, re-export, comportamento, test e fixture. Gli aggregatori top-level già convertiti nel Gruppo 2 restano parte della struttura di destinazione.

Nota di pianificazione emersa dall'ispezione iniziale: la mappa assegna `content/database.rs` a `content/database/mod.rs` e `content/database/tests.rs` a `content/database/tests/mod.rs`, ma i due wrapper risultano ancora nelle posizioni legacy. Il parent dei test contiene fixture condivise e quattro test inline, tre dei quali costituiscono un gruppo coerente di validazione dei talenti; la mappa include quindi `talent_validation_tests.rs` per separare quel gruppo mantenendo le fixture comuni e il test singolo sulle dipendenze dei pacchetti nel parent. La mappa è aggiornata prima degli spostamenti.

L'audit comparativo ha inoltre rilevato che gli helper di movimento di Player e Navigation duplicano gli stessi algoritmi di collisione, discesa, gravità e step-up con costanti e nomi specifici per dominio. Poiché valori e corpi sono equivalenti, il piano introduce `src/movement/mod.rs` come modulo interno condiviso; i wrapper Player/Enemy conserveranno i nomi locali, i valori e i test esistenti tramite re-export, mentre la registrazione del modulo sarà privata in `main.rs`. Questo collegamento cross-domain è necessario per eliminare la duplicazione senza cambiare le formule.

Esito del gruppo: i wrapper Content sono stati convertiti e i tre test di validazione talenti separati; gli helper Player/Enemy ora delegano al modulo interno condiviso mantenendo alias, costante `JUMP_SPEED` specifica del player e test inline. Character, Player, Navigation, Fellowship e Interaction sono stati valutati secondo il criterio aggiornato; dove le responsabilità erano già coese non è stata introdotta frammentazione. L'inventario statico dei test dei sei domini è invariato (149 occorrenze, 146 nomi distinti). La risoluzione dei moduli, i percorsi dei file, i riferimenti alle fixture e i macro di percorso sono stati controllati; nessun `#[path]`, `include_str!` o `include_bytes!` è presente nei domini coinvolti, salvo due `include_str!` delle fixture Classic già esistenti e con percorsi ancora corretti. `git diff --check` è superato. Nessun controllo Cargo, formattazione, compilazione o test è stato eseguito.

- [x] Convertire i wrapper `content/database.rs` e `content/database/tests.rs` in `database/mod.rs` e `database/tests/mod.rs`; isolare i tre test di validazione dei talenti in `talent_validation_tests.rs`, mantenendo le fixture condivise.
- [x] Auditare il resto di Content (inclusi loader e definizioni per tipo) senza modificare le suddivisioni completate nel Gruppo 4 salvo necessità documentata.
- [x] Auditare Character e i relativi test per responsabilità e fixture condivise; i quattro test formano una suite coerente sulla matrice razza-classe e le fazioni.
- [x] Auditare Player (input, movement e helper) e i collegamenti con combat/navigation.
- [x] Auditare Navigation, mantenendo il plugin registrato una sola volta e i moduli nemici sotto `navigation/enemy/`.
- [x] Centralizzare gli helper di movimento identici di Player/Enemy in `movement/mod.rs`, mantenendo gli alias interni specifici e tutte le suite di test esistenti.
- [x] Valutare Fellowship e Interaction: i rispettivi moduli rappresentano un unico sottosistema membership e un unico flusso pointer hover/click; non sono stati frammentati.
- [x] Verificare `mod`, import, re-export, visibilità, `use super::*`, `#[path]`, `include_str!`, `include_bytes!`, percorsi e test dei domini interessati.
- [x] Confrontare il multinsieme dei nomi dei test prima/dopo (149 occorrenze, 146 nomi distinti); nessuna differenza e fixture condivise mantenute.
- [x] Eseguire `git diff --check`; rinviare formattazione, compilazione e test alla verifica finale tramite GitHub Actions.

### Fase 4 – Audit completo di moduli, import e test

**Stato: COMPLETATO STATICAMENTE — Gruppo 6 concluso sul branch `refactor/src-domain-layout`; la verifica tecnica resta rinviata al Gruppo 7.**

Rilievi iniziali del Gruppo 6: la tabella delle destinazioni non riportava i sei aggregatori combat generati durante il refactoring e registrava due volte il modulo movement condiviso; il checkout iniziale conteneva 133 file Rust. La mappa è stata corretta prima di ulteriori spostamenti. La suite di `targeting/selection` conteneva due gruppi distinti per interazione (selezione/ciclo tramite tastiera e selezione tramite click), con fixture comuni; la mappa è stata aggiornata e i due gruppi sono ora separati, mantenendo le fixture nel parent.

- [x] Cercare ogni dichiarazione `mod` e verificare che punti al file effettivo: 0 destinazioni mancanti nel controllo statico.
- [x] Cercare ogni attributo `#[path = ...]` e aggiornare i percorsi spostati: nessun attributo `#[path]` presente in `src/`.
- [x] Controllare ogni `use super::*`: i moduli parent e le fixture condivise sono stati verificati staticamente; la risoluzione completa dei simboli resta da confermare in compilazione.
- [x] Controllare i re-export `pub use` e `pub(crate) use`; non allargare la visibilità senza motivo: inventario dei 67 re-export combat e 49 Content invariato rispetto a HEAD.
- [x] Controllare `include_str!`, `include_bytes!`, path relativi e riferimenti a file non Rust: tutti i target statici risolvono; nessun `include_bytes!`.
- [x] Verificare che i test esterni e quelli inline siano tutti ancora inclusi con `#[cfg(test)]` e dichiarazioni `mod` esistenti.
- [x] Cercare riferimenti testuali ai vecchi percorsi sotto `src/`: nessun riferimento ai vecchi moduli combat o `enemy_navigation`.
- [x] Confrontare numero e multinsieme dei test con HEAD: 660 occorrenze e 656 nomi distinti in entrambi, nessuna differenza.
- [x] Controllare la lunghezza fisica di ogni file Rust: 135 file, massimo 893 righe (`combat/damage/mitigation.rs`), nessun file oltre 1.000 o 1.500 righe.
- [x] Valutare responsabilità e gruppi di test di tutti i domini a qualunque dimensione; fixture/helper condivisi e assenza di frammentazioni prive di vantaggio verificati. Separati i test tab/click di targeting.

Esito del Gruppo 6 – audit statico complessivo:

- Mappatura: 88/88 percorsi sorgente del baseline sono presenti nella tabella e producono 129 destinazioni operative; sono inoltre registrati 6 aggregatori combat generati. Il totale è 135 file Rust, corrispondenti ai 135 file `.rs` correnti: nessuna destinazione mancante o non esistente.
- Moduli e import: tutte le dichiarazioni `mod` esaminate puntano a un file o `mod.rs` esistente (0 errori statici). Sono stati controllati i `use super::*`, i confini parent e gli import cross-domain. Il controllo statico non sostituisce la compilazione per la risoluzione dei nomi e delle visibilità del compilatore.
- Percorsi: nessun `#[path]`; tutti i `include_str!` risolvono a file presenti e non è presente `include_bytes!`. Nessun riferimento ai vecchi percorsi dei moduli riorganizzati è rimasto sotto `src/`.
- Re-export: confrontati con HEAD i 67 elementi re-esportati da `combat` e i 49 da `content`; nessuna differenza. `navigation` mantiene il re-export richiesto di `EnemyNavigationPlugin`; non sono state introdotte modifiche di visibilità nel Gruppo 6.
- Test: inventario da HEAD e checkout corrente pari a 660 funzioni annotate `#[test]`, 656 nomi distinti e multinsieme identico. La ripartizione registrata nei Gruppi 3–5 è coerente; la suite di targeting conserva tutti i 13 casi (7 tab/ciclo, 6 click), ora in due moduli e con fixture comuni nel parent.
- Modularizzazione: le suite camera, combat, content, character, player, navigation, fellowship e interaction risultano organizzate per dominio/responsabilità secondo i gruppi precedenti. L'audit ha aggiunto soltanto la suddivisione tab/click; gli altri file esaminati sotto 1.000 righe sono rimasti coesi o rappresentano moduli di integrazione/composizione per cui ulteriori file sarebbero frammentazione artificiale. Il file più lungo è `combat/damage/mitigation.rs` a 893 righe; i test restano una suite unitaria di mitigazione e le responsabilità armor/resistance/scuole fanno parte dello stesso dominio.
- Integrità e ordine: il diff di questo gruppo sposta soltanto test e aggiorna dichiarazioni/moduli e documentazione. Le formule e i sistemi di produzione non sono stati modificati. L'ordine dei 14 sistemi in `CombatPlugin` coincide con quello di HEAD; l'ordine dei plugin in `main.rs` è invariato.
- Limite della verifica: non sono stati eseguiti test, build, formattazione o controlli Rust. È confermata la preservazione statica dei test e dei percorsi, non la loro compilazione/esecuzione né la risoluzione completa dei simboli. Le verifiche tecniche sono rinviate al Gruppo 7.
- `git diff --check`: superato; restano solo avvisi Git di conversione LF/CRLF su file preesistenti.

### Fase 5 – Verifica tecnica e chiusura

**Stato: IN PREPARAZIONE — Gruppo 7 autorizzato; il workflow ora applica un controllo di formattazione bloccante. Nessuna verifica tecnica finale è ancora stata eseguita sullo SHA corrente.**

- [x] Rendere bloccante nel workflow `.github/workflows/build.yml` il controllo `cargo fmt --all -- --check`: il comando è eseguito direttamente, quindi un errore fallisce il job.
- [ ] Eseguire il controllo di formattazione nel workflow finale e correggere eventuali problemi rilevati; il workflow non è ancora stato eseguito.
- [ ] Eseguire i controlli di compilazione e test locali disponibili, senza dichiarare la fase conclusa prima dei risultati.
- [ ] Eseguire una sola verifica CI completa dopo aver raggruppato tutte le modifiche strutturali, salvo blocchi che richiedano una diagnosi anticipata.
- [ ] Verificare entrambi i job Linux e Windows e l'avvio headless sullo SHA finale esatto.
- [ ] Verificare che non siano diminuiti i test rispetto al baseline senza una motivazione documentata.
- [ ] Ispezionare il diff per assicurarsi che i cambiamenti siano principalmente spostamenti, aggiornamenti dei moduli e import.
- [ ] Aggiornare la roadmap principale indicando il refactoring come fase preparatoria completata.
- [ ] Registrare qui SHA finale, run CI, numero test e data di completamento.

Preparazione del Gruppo 7: i Gruppi 1–6 sono completati strutturalmente e sottoposti ad audit statico. Il Gruppo 7 è autorizzato e in preparazione. Il controllo `cargo fmt --all -- --check` è ora bloccante, ma non è stato eseguito. Formattazione sullo SHA finale, compilazione, suite completa dei test e GitHub Actions restano da verificare; nessun controllo tecnico finale è superato sullo SHA corrente. Il workflow esegue inoltre `cargo check --all-targets`, build e `cargo test --release` su Linux/Windows e avvio headless su entrambe le piattaforme. Il trigger push riguarda soltanto `main`, il trigger PR soltanto PR verso `main` e `workflow_dispatch` consente avvio manuale. La verifica remota è ancora da avviare e il workflow carica artifact; non è stato avviato.

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
│   │   │   │   └── mod.rs
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
```

## Registro completo degli 88 file (percorso attuale → destinazione)

| Percorso attuale | Percorso proposto |
|---|---|
| `src/camera.rs` | `src/camera/mod.rs`, `src/camera/tests/mod.rs`, `src/camera/tests/controls_tests.rs`, `src/camera/tests/follow_tests.rs`, `src/camera/tests/visibility_tests.rs` |
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
| `src/combat/effect_dispatcher/tests.rs` | `src/combat/effects/dispatcher/tests/mod.rs`, `src/combat/effects/dispatcher/tests/subscriber_tests.rs`, `src/combat/effects/dispatcher/tests/reactive_runtime_tests.rs` |
| `src/combat/effect_dispatcher/tests/area_tests.rs` | `src/combat/effects/dispatcher/tests/area_tests.rs` |
| `src/combat/effect_execution.rs` | `src/combat/effects/execution/mod.rs` (implementazione), `src/combat/effects/execution/tests/mod.rs` (test inline estratti) |
| `src/combat/effect_magnitude.rs` | `src/combat/effects/magnitude.rs` |
| `src/combat/effect_resolution.rs` | `src/combat/effects/resolution.rs` |
| `src/combat/effect_runtime.rs` | `src/combat/effects/runtime/mod.rs` |
| `src/combat/effect_runtime/aura_tests.rs` | `src/combat/effects/runtime/tests/aura_tests/mod.rs`, `src/combat/effects/runtime/tests/aura_tests/operation_tests.rs`, `src/combat/effects/runtime/tests/aura_tests/lifecycle_tests.rs`, `src/combat/effects/runtime/tests/aura_tests/reactive_tests.rs` |
| `src/combat/effect_runtime/corruption_tests.rs` | `src/combat/effects/runtime/tests/corruption_tests.rs` |
| `src/combat/effect_runtime/custom_effect_tests.rs` | `src/combat/effects/runtime/tests/custom_effect_tests.rs` |
| `src/combat/effect_runtime/displacement_tests.rs` | `src/combat/effects/runtime/tests/displacement_tests.rs` |
| `src/combat/effect_runtime/periodic_tests.rs` | `src/combat/effects/runtime/tests/periodic_tests.rs` |
| `src/combat/effect_runtime/tests.rs` | `src/combat/effects/runtime/tests/mod.rs`, `src/combat/effects/runtime/tests/queue_tests.rs`, `src/combat/effects/runtime/tests/execution_tests.rs` |
| `src/combat/effect_targeting.rs` | `src/combat/effects/targeting.rs` |
| `src/combat/enemy_ai.rs` | `src/combat/enemy/ai/mod.rs` |
| `src/combat/enemy_ai/tests.rs` | `src/combat/enemy/ai/tests/mod.rs`, `src/combat/enemy/ai/tests/target_acquisition_tests.rs`, `src/combat/enemy/ai/tests/aggro_lifecycle_tests.rs` |
| `src/combat/enemy_attacks.rs` | `src/combat/enemy/attacks/mod.rs` |
| `src/combat/enemy_attacks/tests.rs` | `src/combat/enemy/attacks/tests/mod.rs`, `src/combat/enemy/attacks/tests/eligibility_tests.rs`, `src/combat/enemy/attacks/tests/damage_resolution_tests.rs` |
| `src/combat/events.rs` | `src/combat/core/events/mod.rs`, `src/combat/core/events/tests/mod.rs`, `src/combat/core/events/tests/context_validation_tests.rs`, `src/combat/core/events/tests/queue_tests.rs`, `src/combat/core/events/tests/subscription_tests.rs` |
| `src/combat/healing.rs` | `src/combat/core/healing.rs` |
| `src/combat/health.rs` | `src/combat/core/health.rs` |
| `src/combat/player_attacks.rs` | `src/combat/player/attacks/mod.rs` |
| `src/combat/player_attacks/tests.rs` | `src/combat/player/attacks/tests/mod.rs`, `src/combat/player/attacks/tests/eligibility_tests.rs`, `src/combat/player/attacks/tests/ability_resolution_tests.rs` |
| `src/combat/resources.rs` | `src/combat/core/resources.rs` |
| `src/combat/stats.rs` | `src/combat/core/stats.rs` |
| `src/combat/summon_ai.rs` | `src/combat/summon/ai.rs` |
| `src/combat/summon_attacks.rs` | `src/combat/summon/attacks.rs` |
| `src/combat/summon_integration_tests.rs` | `src/combat/summon/tests/mod.rs`, `src/combat/summon/tests/spawn_tests.rs`, `src/combat/summon/tests/combat_tests.rs`, `src/combat/summon/tests/lifecycle_tests.rs`, `src/combat/summon/tests/classic_pack_tests.rs` |
| `src/combat/summon_lifecycle.rs` | `src/combat/summon/lifecycle.rs` |
| `src/combat/summons.rs` | `src/combat/summon/mod.rs`, `src/combat/summon/unit_tests/mod.rs`, `src/combat/summon/unit_tests/types_tests.rs`, `src/combat/summon/unit_tests/spawn_tests.rs`, `src/combat/summon/unit_tests/runtime_tests.rs` |
| `src/combat/target_indicator.rs` | `src/combat/targeting/indicator/mod.rs` |
| `src/combat/target_indicator/tests.rs` | `src/combat/targeting/indicator/tests/mod.rs` |
| `src/combat/target_selection.rs` | `src/combat/targeting/selection/mod.rs` |
| `src/combat/target_selection/tests.rs` | `src/combat/targeting/selection/tests/mod.rs` (shared fixture), `src/combat/targeting/selection/tests/tab_tests.rs`, `src/combat/targeting/selection/tests/click_tests.rs` |
| `src/content.rs` | `src/content/mod.rs` |
| `src/content/ability.rs` | `src/content/ability.rs` |
| `src/content/aura.rs` | `src/content/aura.rs` |
| `src/content/combat_attribute.rs` | `src/content/combat_attribute.rs` |
| `src/content/creature.rs` | `src/content/creature.rs` |
| `src/content/damage_school.rs` | `src/content/damage_school.rs` |
| `src/content/database.rs` | `src/content/database/mod.rs` |
| `src/content/database/lookup.rs` | `src/content/database/lookup.rs` |
| `src/content/database/registration.rs` | `src/content/database/registration.rs` |
| `src/content/database/tests.rs` | `src/content/database/tests/mod.rs`, `src/content/database/tests/talent_validation_tests.rs` |
| `src/content/database/tests/aura_tests.rs` | `src/content/database/tests/aura_tests.rs` |
| `src/content/database/tests/classic_pack_tests.rs` | `src/content/database/tests/classic_pack_tests.rs` |
| `src/content/database/tests/reference_tests.rs` | `src/content/database/tests/reference_tests.rs` |
| `src/content/database/tests/registration_tests.rs` | `src/content/database/tests/registration_tests.rs` |
| `src/content/database/validation.rs` | `src/content/database/validation.rs` |
| `src/content/effect.rs` | `src/content/effect.rs` |
| `src/content/id.rs` | `src/content/id.rs` |
| `src/content/loader.rs` | `src/content/loader.rs`, `src/content/loader/tests/mod.rs`, `src/content/loader/tests/serialization_tests.rs`, `src/content/loader/tests/registration_tests.rs`, `src/content/loader/tests/validation_tests.rs` |
| `src/content/metadata.rs` | `src/content/metadata.rs` |
| `src/content/pack.rs` | `src/content/pack.rs` |
| `src/content/registry.rs` | `src/content/registry.rs` |
| `src/content/resource.rs` | `src/content/resource.rs` |
| `src/content/talent.rs` | `src/content/talent.rs` |
| `src/enemy_navigation.rs` | `src/navigation/enemy/mod.rs` |
| `src/enemy_navigation/movement.rs` | `src/navigation/enemy/movement.rs` |
| `src/enemy_navigation/movement_helpers.rs` | `src/navigation/enemy/movement_helpers.rs` (facade interna e test; condivide l'implementazione comune registrata nella riga del helper Player) |
| `src/enemy_navigation/targets.rs` | `src/navigation/enemy/targets.rs` |
| `src/fellowship.rs` | `src/fellowship/mod.rs` |
| `src/interaction.rs` | `src/interaction/mod.rs` |
| `src/main.rs` | `src/main.rs` |
| `src/navigation.rs` | `src/navigation/mod.rs` |
| `src/player.rs` | `src/player/mod.rs` |
| `src/player/input.rs` | `src/player/input.rs` |
| `src/player/movement.rs` | `src/player/movement.rs` |
| `src/player/movement_helpers.rs` | `src/player/movement_helpers.rs` (facade interna, costante jump e test), `src/movement/mod.rs` (algoritmi condivisi) |

## Regole architetturali

### Moduli aggregatori aggiunti dalla riorganizzazione

Questi moduli sono stati creati per rendere esplicita la gerarchia dei domini e non corrispondono a file dell'inventario originario degli 88 sorgenti:

| Origine | Destinazione finale | Responsabilità |
|---|---|---|
| Generato nel Gruppo 3 | `src/combat/aura/mod.rs` | Aggregatore del dominio aura. |
| Generato nel Gruppo 3 | `src/combat/core/mod.rs` | Aggregatore degli eventi, salute, cure, risorse e statistiche combat. |
| Generato nel Gruppo 3 | `src/combat/effects/mod.rs` | Aggregatore del runtime degli effetti. |
| Generato nel Gruppo 3 | `src/combat/enemy/mod.rs` | Aggregatore AI e attacchi nemici. |
| Generato nel Gruppo 3 | `src/combat/player/mod.rs` | Aggregatore degli attacchi del player. |
| Generato nel Gruppo 3 | `src/combat/targeting/mod.rs` | Aggregatore di indicator e selezione target. |

1. `src/main.rs` resta il punto d'ingresso; gli altri moduli di primo livello diventano directory con `mod.rs`.
2. Ogni dominio ha una propria directory: personaggi, combattimento, contenuti, navigazione, giocatore, fellowship e interazione.
3. Dentro `combat/`, i sottosistemi sono separati per responsabilità: `aura`, `damage`, `death`, `displacement`, `effects`, `enemy`, `player`, `summon`, `targeting` e `core`.
4. I test esterni sono raccolti nella directory `tests/` del rispettivo sistema. I test inline sono stati estratti quando formavano gruppi di responsabilità distinti; gli altri possono restare accanto all'implementazione quando la suite è coesa e l'estrazione non offre un vantaggio concreto.
5. I sei `mod.rs` aggiuntivi sotto `combat/` sono aggregatori nuovi, necessari per esporre ordinatamente i sottosistemi: `aura`, `core`, `effects`, `enemy`, `player` e `targeting`.
6. I file di contenuto `.ron` e la configurazione Cargo/workflow non vengono spostati in questa riorganizzazione.

## 8. Registro di avanzamento

| Area | Stato | Note / commit |
|---|---|---|
| Fase 0 – Preparazione | IN CORSO | Branch `refactor/src-domain-layout` creato dallo SHA `454874a...`; controlli di pulizia e conferma branch attivo ancora da completare. |
| Fase 1 – Moduli top-level | STRUTTURA COMPLETATA | 13 percorsi convertiti/spostati; wiring e audit statico completati. Verifiche Rust rinviate alla verifica finale tramite GitHub Actions. |
| Fase 2 – Dominio combat | STRUTTURA COMPLETATA | Mappa, split, moduli, percorsi, re-export, test e ordine dei sistemi verificati staticamente; `git diff --check` superato. Formattazione, compilazione e test rinviati alla verifica finale tramite GitHub Actions. |
| Fase 3 / Gruppo 5 – Altri domini | STRUTTURA COMPLETATA E VERIFICATA STATICAMENTE | Content, Character, Player, Navigation, Fellowship e Interaction; inventario test invariato (149 occorrenze, 146 nomi distinti), percorsi/moduli verificati, `git diff --check` superato. Verifiche Rust rinviate a GitHub Actions. |
| Gruppo 4 – Suddivisione per responsabilità | COMPLETATO STATICAMENTE | Test e moduli suddivisi secondo la mappa; inventario dei 159 test invariato, percorsi dei moduli verificati e `git diff --check` superato. Formattazione, compilazione e test rinviati alla verifica finale tramite GitHub Actions. |
| Fase 4 / Gruppo 6 – Audit strutturale complessivo | COMPLETATO STATICAMENTE | 135 file mappati; 0 moduli o percorsi mancanti; test 660/660 invariati rispetto a HEAD; re-export combat/content invariati; massimo 893 righe; split test tab/click; `git diff --check` superato. Risoluzione compilatore e verifiche tecniche rinviate al Gruppo 7. |
| Fase 5 – Verifica e chiusura (Gruppo 7) | IN PREPARAZIONE | Autorizzato; controllo rustfmt bloccante predisposto ma non eseguito. Formattazione finale, compilazione, test e GitHub Actions non eseguiti sullo SHA corrente. |

### Registro delle decisioni e delle eccezioni

| Data | Decisione / modifica al piano | Motivazione | Stato |
|---|---|---|---|
| 2026-11-07 | Creato il piano iniziale con inventario completo di 88 file. | Preparare la riorganizzazione prima di iniziare la Fase 4 del progetto. | REGISTRATO |
| 2026-11-07 | Creato `refactor/src-domain-layout` dallo SHA `454874a358a18e7e565043d797e3c42ddc46323f`; `main` resta intatto. | Isolare tutti gli spostamenti e le modifiche strutturali dal branch stabile. | REGISTRATO |
| 2026-11-07 | Aggiunta la regola dimensionale: obiettivo massimo 1.000 righe per file, 1.500 soltanto in casi eccezionali; censiti i quattro file oltre 1.000 e definita la loro suddivisione. | La riorganizzazione deve anche ridurre i file monolitici, non limitarsi a spostarli. | REGISTRATO |
| 2026-11-08 | Avviato il Gruppo 2: convertiti i moduli top-level previsti e integrata enemy navigation sotto `navigation`; plugin re-esportato da `navigation` e import aggiornato in `main.rs`. | Ridurre i moduli top-level mantenendo plugin e comportamento invariati. La verifica `cargo fmt --check` resta pendente perché `cargo` non è installato o disponibile nel PATH dell'ambiente. | IN CORSO |
| 2026-11-08 | Gruppo 2 dichiarato strutturalmente completato su autorizzazione dell'utente; Gruppo 3 combat completato con audit statico. | Le verifiche tecniche Rust (formattazione, compilazione e test) sono state esplicitamente rinviate alla verifica finale tramite GitHub Actions. `git diff --check` superato; nessun test/build eseguito. | COMPLETATO STATICAMENTE |
| 2026-11-08 | Criterio di modularizzazione aggiornato e sostitutivo della precedente regola dimensionale per tutti i domini: responsabilità e coesione guidano la suddivisione a qualunque dimensione; 1.000 righe è una soglia di attenzione. Test raggruppati per funzione con fixture/helper condivisi. | Estendere la modularizzazione utile anche ai file brevi, evitando sia duplicazioni sia frammentazione artificiale. Applicazione ai gruppi ancora da completare e all'audit complessivo; nessun file Rust modificato da questa decisione documentale. | REGISTRATO |
| 2026-11-08 | Avviato il Gruppo 4 dopo aver verificato branch, HEAD e working tree; mappate le separazioni per responsabilità emerse dall'audit statico. | Rendere i test navigabili per comportamento e separare suite eterogenee a qualunque dimensione, senza modificare il comportamento. | IN CORSO |
| 2026-11-08 | Completate le suddivisioni strutturali del Gruppo 4: test di camera, combat events/summon, runtime aura, dispatcher, enemy AI/attacks, player attacks e content loader separati per responsabilità; helper condivisi mantenuti nei parent. | Applicare il criterio di modularizzazione aggiornato mantenendo invariati test e comportamento. Audit statico dei percorsi e dei nomi di test superato; `git diff --check` senza errori. Formattazione, compilazione e test non eseguiti, rinviati alla verifica finale. | COMPLETATO STATICAMENTE |
| 2026-11-08 | Avviato il Gruppo 5 dopo aver riletto i documenti e verificato branch, HEAD e working tree. | Riordinare Content, Character, Player, Navigation, Fellowship e Interaction secondo responsabilità, conservando integralmente le modifiche precedenti dei Gruppi 2–4. | REGISTRATO |
| 2026-11-08 | Completato strutturalmente il Gruppo 5: convertiti i wrapper database Content, separati i test di validazione talenti e centralizzati gli helper duplicati Player/Enemy; gli altri domini sono stati auditati senza frammentazioni non necessarie. | Applicare il criterio di modularizzazione aggiornato preservando API, comportamento, fixture e test. Inventario test statico invariato (149 occorrenze, 146 nomi distinti), moduli e percorsi verificati, `git diff --check` superato. Formattazione, compilazione e test rinviati a GitHub Actions. | COMPLETATO STATICAMENTE |
| 2026-11-08 | Avviato il Gruppo 6 dopo rilettura integrale dei documenti e controllo del branch, HEAD e working tree. | Audit complessivo di mappa, moduli, import/re-export, visibilità, percorsi, test, fixture, dimensioni e coesione in tutto `src/`; verifiche tecniche Rust rinviate al Gruppo 7. | IN CORSO |
| 2026-11-08 | Completato l'audit statico complessivo: corretta la mappa degli aggregatori generati e del helper movement condiviso; test di targeting separati per input tab/click. | Rendere la mappa esatta rispetto ai 135 file correnti, conservare test e fixture, controllare import/re-export e responsabilità in tutto `src/`. Baseline e checkout: 660 test / 656 nomi distinti identici; moduli e percorsi senza errori statici; `git diff --check` superato. Compilazione e test rinviati al Gruppo 7. | COMPLETATO STATICAMENTE |

## 9. Criteri di completamento

Il refactoring è concluso soltanto quando tutte le condizioni seguenti sono vere:

- [ ] Tutti gli 88 file originali sono stati ricollocati secondo il registro oppure hanno una destinazione alternativa documentata.
- [ ] Nessun file sorgente o test è andato perso.
- [ ] Tutti i moduli vengono risolti correttamente e tutti gli import sono aggiornati.
- [ ] Le interfacce pubbliche e `pub(crate)` necessarie sono mantenute.
- [ ] Tutti i test vengono ancora compilati ed eseguiti.
- [ ] Ogni file oltre 1.000 righe è stato valutato e suddiviso quando le responsabilità sono separabili, oppure la scelta di mantenerlo intero è motivata; nessun file supera 1.500 righe senza approvazione esplicita e motivazione documentata.
- [ ] Tutti i moduli e i test, in ogni dominio e a qualunque dimensione, sono stati valutati per responsabilità separabili; le suddivisioni utili sono state applicate, le fixture/helper condivisi non sono duplicati e non sono state introdotte frammentazioni artificiali.
- [ ] Il comportamento dei sistemi non è stato modificato intenzionalmente.
- [ ] CI Linux, CI Windows e avvio headless risultano verdi sullo SHA finale.
- [ ] La roadmap principale riporta il completamento del passaggio preparatorio.

## 10. Stato attuale

**Gruppo 1 completato e verificato; Gruppi 2–6 completati strutturalmente e sottoposti ad audit statico. Gruppo 7 autorizzato e in fase di preparazione.** Audit del Gruppo 6: mappatura esatta di 135 file, test invariati (660 occorrenze/656 nomi distinti), re-export controllati, nessun percorso o modulo mancante, massimo 893 righe e `git diff --check` superato. Formattazione, compilazione, esecuzione test e risoluzione completa dei nomi restano da verificare: nessuna verifica tecnica finale è ancora superata sullo SHA corrente. Il lavoro resta sul branch `refactor/src-domain-layout`; nessun commit o aggiornamento remoto è stato eseguito.
