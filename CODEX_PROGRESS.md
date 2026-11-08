# Registro di avanzamento Codex

## Identificazione e checkpoint iniziale

- Progetto: `WoW-Friendship-Crawler` (`konakona185-gif/wow-friendship-crawler`).
- Branch di lavoro esclusivo: `refactor/src-domain-layout`.
- SHA iniziale della preparazione: `6989ae52c59b5e17264364c9926949e10aacf50a`.
- Stato Git iniziale: working tree pulito; branch allineato a `origin/refactor/src-domain-layout`.
- SHA iniziale di creazione del branch, riportato nei documenti progettuali: `454874a358a18e7e565043d797e3c42ddc46323f`.
- Fase attuale: Gruppi 1–6 della Fase 3.5 completati strutturalmente e sottoposti ad audit statico; Gruppo 7 autorizzato e in fase di preparazione. Formattazione, compilazione, test e GitHub Actions sono ancora da eseguire e nessuna verifica tecnica finale è stata superata sullo SHA corrente. La Fase 4 resta bloccata fino alla verifica finale.
- Ultimo checkpoint Git committato: `a4bee565e306f630fbca5688a23cf891f658508c`.

## Stato verificato all'avvio

- Ricognizione di `src/`, dei test, delle dimensioni e delle dipendenze completata il 2026-11-08.
- Inventario osservato: 88 file Rust in 18 directory; dimensione misurata nella ricognizione: 948.467 byte.
- Riconciliazione dimensionale verificata: i blob Git della baseline iniziale occupano 913.840 byte; con `core.autocrlf=true`, il checkout Windows converte 34.627 terminazioni LF in CRLF, per un totale sul disco di 948.467 byte (913.840 + 34.627). Tutti gli 88 file Rust contribuiscono alla differenza e i sorgenti risultano identici alla baseline.
- Mappatura del piano operativo verificata: 88 percorsi sorgente corrispondenti ai file attuali, 98 destinazioni uniche, nessun sorgente mancante o non mappato e nessuna destinazione duplicata. Le 10 destinazioni aggiuntive derivano dalla suddivisione pianificata dei quattro file di test più grandi.
- Quattro file superano 1.000 righe nella ricognizione: `combat/aura_runtime/tests.rs` (1.303), `combat/effect_execution.rs` (1.152), `combat/summon_integration_tests.rs` (1.137) e `combat/effect_runtime/tests.rs` (1.007).
- La ricognizione non ha modificato il codice né eseguito test.
- I documenti permanenti `AGENTS.md` e `CODEX_PROGRESS.md` sono stati creati in questa attività e verificati dopo la scrittura.

## Checkpoint documentale del 2026-11-08

- Branch attivo: `refactor/src-domain-layout`.
- HEAD all'inizio dell'attività: `eae9af014ce1fbd935bc1f6f41b37109cca57447`.
- Stato Git all'inizio: working tree pulito; branch avanti di un commit rispetto a `origin/refactor/src-domain-layout`.
- Il commit locale `eae9af014ce1fbd935bc1f6f41b37109cca57447` contiene esclusivamente `AGENTS.md` e `CODEX_PROGRESS.md`.
- Stato Git dopo le modifiche documentali di questa attività: working tree con modifiche non committate esclusivamente in `.github/workflows/benilla-classic-pipeline-revised-v3.md` e `CODEX_PROGRESS.md`; nessuna modifica a `AGENTS.md` o ai file Rust.
- Ultimo checkpoint Git committato: `eae9af014ce1fbd935bc1f6f41b37109cca57447`.
- Le modifiche documentali richieste in questa attività sono state rilette e confrontate nel diff. Nessun test o build eseguito.

## Gruppo 1 – Preparazione: COMPLETATO E VERIFICATO

Verifica conclusiva del 2026-11-08, eseguita sul branch `refactor/src-domain-layout` allo SHA `a4bee565e306f630fbca5688a23cf891f658508c`:

- Baseline Git riconciliata: i blob di `src/` alla revisione iniziale `454874a358a18e7e565043d797e3c42ddc46323f` totalizzano 913.840 byte. Con `core.autocrlf=true`, le 34.627 terminazioni LF convertite in CRLF portano i file del checkout Windows a 948.467 byte. L'equazione verificata è 913.840 + 34.627 = 948.467 byte.
- Inventario verificato: 88 file Rust in 18 directory sotto `src/`.
- Mappatura verificata: 88 percorsi sorgente nel registro, tutti presenti nell'albero attuale; 98 destinazioni uniche, zero sorgenti mancanti o non mappate e zero destinazioni duplicate.
- Dimensioni verificate contando le righe fisiche: superano 1.000 righe `src/combat/aura_runtime/tests.rs` (1.303), `src/combat/effect_execution.rs` (1.152), `src/combat/effect_runtime/tests.rs` (1.007) e `src/combat/summon_integration_tests.rs` (1.137). Nessun file supera 1.500 righe.
- Il contenuto Git di `src/` è identico alla baseline iniziale; nessun file Rust è stato modificato.
- Stato Git prima di registrare questo completamento: working tree pulito, branch avanti di due commit rispetto a `origin`; HEAD `a4bee565e306f630fbca5688a23cf891f658508c`.
- Stato Git dopo questo aggiornamento: l'unica modifica non committata prevista è `CODEX_PROGRESS.md`; nessuna modifica a sorgenti Rust. Nessun test o build è stato eseguito.

Esito: tutti i controlli di preparazione richiesti sono stati confrontati con il piano operativo e risultano coerenti. Il Gruppo 1 è completo e verificato; il Gruppo 2 non è iniziato.

## Gruppo 2 – Conversione dei moduli top-level e enemy navigation: STRUTTURA COMPLETATA

Checkpoint di avvio del 2026-11-08:

- Branch: `refactor/src-domain-layout`.
- HEAD: `a4bee565e306f630fbca5688a23cf891f658508c`.
- Stato Git all'avvio: l'unica modifica presente era la modifica non committata già documentata a `CODEX_PROGRESS.md`, relativa alla chiusura del Gruppo 1; nessun altro file risultava modificato.
- Moduli da convertire secondo la mappa: `camera.rs`, `character.rs`, `combat.rs`, `content.rs`, `fellowship.rs`, `interaction.rs`, `navigation.rs` e `player.rs` in directory rispettive con `mod.rs`.
- Enemy navigation da integrare: `enemy_navigation.rs` e i suoi tre moduli figli sotto `navigation/enemy/`; aggiornamento del wiring in `main.rs` per importare il plugin dal dominio `navigation`.
- Vincoli: preservare simboli e API esistenti, test, import e registrazione del plugin; nessuna modifica funzionale. Verifiche richieste a fine gruppo: `git diff --check` e `cargo fmt --check`; nessuna suite completa né GitHub Actions.
- Nessun file Rust era ancora stato modificato al momento di questo checkpoint.

### Attività eseguite e controlli

- Spostati i moduli top-level secondo la mappa: `camera.rs`, `character.rs`, `combat.rs`, `content.rs`, `fellowship.rs`, `interaction.rs`, `navigation.rs` e `player.rs` nelle rispettive directory `mod.rs`.
- Spostati `character/tests.rs` in `character/tests/mod.rs` e i quattro file di enemy navigation (`enemy_navigation.rs`, `movement.rs`, `movement_helpers.rs`, `targets.rs`) sotto `navigation/enemy/`.
- `navigation/mod.rs` dichiara `mod enemy` e ri-esporta `EnemyNavigationPlugin`; `main.rs` rimuove il modulo top-level `enemy_navigation`, importa entrambi i plugin da `navigation` e conserva l'ordine di registrazione.
- Verificata l'assenza dei vecchi percorsi, la presenza delle nuove destinazioni e l'identità dei contenuti per i 12 file spostati senza modifiche. Il corpo di `navigation.rs` è stato confrontato con la nuova `navigation/mod.rs`, al netto delle due dichiarazioni di modulo/re-export aggiunte.
- `git diff --check`: superato (nessun errore; Git segnala solo la normale conversione LF/CRLF di `CODEX_PROGRESS.md`).
- `cargo fmt --check`: tentato ma non eseguibile; `cargo` e `rustfmt` non sono disponibili nel PATH e non risultano installati nei percorsi Rust controllati.
- Ricontrollo disponibilità strumenti del 2026-11-08: `cargo --version`, `rustfmt --version` e `rustup show` non possono essere eseguiti perché i rispettivi comandi non sono riconosciuti in PowerShell. `C:\Users\Admin\.cargo\bin` non esiste e quindi non contiene `cargo.exe`, `rustfmt.exe`, `rustup.exe` o `rustc.exe`; anche `C:\Users\Admin\.rustup\toolchains` non esiste. Non è stato possibile individuare un eseguibile Cargo da invocare per percorso assoluto; `rustfmt` non è disponibile come alternativa. Non installati strumenti e non modificate configurazioni dell'ambiente.
- Audit statico del Gruppo 2 (2026-11-08): superato per struttura e contenuti. Confrontate tutte le 13 destinazioni della mappa del Gruppo 2; nessuna vecchia posizione esiste ancora. Hash Git dei 12 file trasferiti senza modifiche identici agli originali; il contenuto di `navigation/mod.rs` coincide con `navigation.rs` dopo aver escluso le sole dichiarazioni aggiunte per `enemy` e il re-export. Tutte le dichiarazioni `mod` nei file coinvolti risolvono a un file presente.
- L'audit non ha trovato riferimenti obsoleti al modulo/percorso `enemy_navigation`; i nomi dei sistemi che contengono la stringa `enemy_navigation` restano parte del codice spostato. `EnemyNavigationPlugin` è dichiarato in `navigation::enemy`, ri-esportato da `navigation` e importato da `main.rs`. Il diff di `main.rs` mostra solo la rimozione del vecchio modulo top-level e l'aggiornamento dell'import: ordine e chiamate di registrazione dei plugin sono invariati.
- Controllati tutti i riferimenti `#[path]`, `include_str!` e `include_bytes!` sotto `src/`: tutti i target esistono. I file di test trasferiti sono identici agli originali; in particolare `character/tests.rs` conserva 4 test e `enemy_navigation/targets.rs` conserva 11 test. Non risultano test rimossi o ridotti. Le sole differenze Rust osservate sono strutturali (spostamenti, dichiarazione/re-export del modulo e import in `main.rs`); l'assenza di cambiamenti funzionali è valutata staticamente, non tramite esecuzione.
- `git diff --check` ripetuto durante l'audit: superato, senza errori (resta l'avviso Git di conversione LF/CRLF per `CODEX_PROGRESS.md`).
- Nessun test, build, GitHub Actions o commit eseguito.
- Formattazione non verificata: Cargo e rustfmt sono indisponibili. `cargo fmt --check`, compilazione e test sono rinviati alla verifica finale tramite GitHub Actions. Su decisione dell'utente, il lavoro strutturale del Gruppo 2 è considerato COMPLETATO; le verifiche tecniche restano esplicitamente pendenti e non sono dichiarate superate.

### Moduli spostati e wiring modificato

- `src/camera.rs` → `src/camera/mod.rs`
- `src/character.rs` → `src/character/mod.rs`
- `src/character/tests.rs` → `src/character/tests/mod.rs`
- `src/combat.rs` → `src/combat/mod.rs`
- `src/content.rs` → `src/content/mod.rs`
- `src/fellowship.rs` → `src/fellowship/mod.rs`
- `src/interaction.rs` → `src/interaction/mod.rs`
- `src/navigation.rs` → `src/navigation/mod.rs`
- `src/enemy_navigation.rs` → `src/navigation/enemy/mod.rs`
- `src/enemy_navigation/movement.rs` → `src/navigation/enemy/movement.rs`
- `src/enemy_navigation/movement_helpers.rs` → `src/navigation/enemy/movement_helpers.rs`
- `src/enemy_navigation/targets.rs` → `src/navigation/enemy/targets.rs`
- `src/player.rs` → `src/player/mod.rs`
- Modificato `src/main.rs`: rimosso `mod enemy_navigation;`, aggiornato l'import di `EnemyNavigationPlugin` da `navigation`.
- Modificato `src/navigation/mod.rs`: aggiunti il modulo `enemy` e il re-export pubblico del plugin.
- Modificati i documenti di stato `CODEX_PROGRESS.md` e `.github/workflows/refactoring.md`.

## Registro delle modifiche

### Completate e verificate

- Creazione di `AGENTS.md` con i vincoli permanenti di branch, sicurezza Git, ambito strutturale, lettura obbligatoria dei documenti, soglie dimensionali e continuità. Verificata rileggendo il file scritto.
- Creazione di `CODEX_PROGRESS.md` con il checkpoint iniziale, la ricognizione, i gruppi previsti, le regole di verifica e la procedura di ripresa. Verificata rileggendo il file scritto.
- Gruppo 1 – Preparazione completato e verificato: baseline byte Git/checkout riconciliata; inventario 88/88 file verificato; mappatura 88 sorgenti verso 98 destinazioni verificata senza mancanti o duplicati; audit dimensionale confermato; sorgenti confrontati con la baseline senza differenze.

### Scritte ma non verificate

- Nessuna.

### Incomplete

- Formattazione, compilazione e test dei Gruppi 2 e 3: non eseguiti e rinviati alla verifica finale tramite GitHub Actions.
- Gruppi 2–6: strutturalmente completati e verificati staticamente secondo i checkpoint finali riportati nelle sezioni dedicate; formattazione, compilazione e test restano non eseguiti.
- Gruppo 6 – Audit strutturale complessivo: COMPLETATO E VERIFICATO STATICAMENTE nel checkpoint del 2026-11-08 riportato in fondo al registro.
- Gruppo 7 – Preparazione della verifica tecnica finale: autorizzato e in preparazione; nessun controllo tecnico né GitHub Actions è stato avviato.

## Gruppi di lavoro previsti

1. Preparazione: ricontrollare branch e working tree, riconciliare la baseline, verificare inventario e dimensioni prima degli spostamenti.
2. Conversione dei moduli top-level in directory con `mod.rs` e integrazione della enemy navigation.
3. Riordino del dominio combat per aree coerenti, preservando API, test e comportamento.
4. Valutazione e suddivisione dei moduli e dei test in tutti i domini in base alle responsabilità, a qualunque dimensione; raggruppare test per funzionalità, mantenere fixture/helper condivisi e non creare frammentazioni artificiali.
5. Riordino di content, character, player, navigation, fellowship e interaction.
6. Audit completo di moduli, import, re-export, percorsi, test, dimensioni e coesione architetturale in tutti i domini, inclusi file sotto la soglia delle 1.000 righe.
7. Verifica finale prevista dalla roadmap: formattazione, compilazione/test locali disponibili e CI Linux/Windows/headless sullo SHA finale.

Ogni gruppo richiede un controllo dello stato Git prima di iniziare e un aggiornamento di questo registro dopo il lavoro. I commit locali di sicurezza richiedono una richiesta di autorizzazione prima di essere creati. Push, merge e Pull Request richiedono autorizzazione esplicita.

## Test e verifiche

### Superati

- Nessun test è stato eseguito in questa attività.
- Nessun test o build è stato eseguito durante l'aggiornamento documentale del 2026-11-08.
- Nessun test o build è stato eseguito per il Gruppo 1, che consisteva in verifiche in lettura e registrazione documentale.
- Nessun test o build è stato eseguito per il Gruppo 2, come richiesto. L'audit statico è superato e `git diff --check` è superato; `cargo fmt --check` non è stato completabile per indisponibilità di Cargo/rustfmt ed è rinviato alla verifica finale.
- Gruppo 3 – Riordino combat (2026-11-08): `git diff --check` superato senza errori (Git segnala solo la conversione LF/CRLF prevista per `CODEX_PROGRESS.md`). Audit statico: mappatura delle destinazioni verificata; 40 file spostati senza modifiche hanno mantenuto hash Git identico; dichiarazioni di modulo e riferimenti di percorso controllati; 496 funzioni di test prima e dopo; 67 re-export pubblici confrontati senza differenze; ordine di registrazione dei sistemi invariato; nessun file Rust in `src/combat/` supera 1.000 righe. Nessun test, build o controllo Rust eseguito.
- La documentazione riporta come riferimento storico la Run CI #522: 660 test superati su Linux e Windows e avvio headless verificato sullo SHA `55e001ec7dc06f5d9bbb3107f3b5ef3f96354b38`. Non è una verifica dell'attuale SHA.

### Falliti

- Nessun test eseguito; quindi nessun test fallito in questa attività.

### Non eseguiti / da eseguire

- Tutti i test e le verifiche del refactoring sono da eseguire dopo i gruppi di modifiche, secondo il piano e la CI finale prevista.

## Regole per checkpoint e ripresa

Prima di ogni gruppo:

1. Verifica `git branch --show-current`, `git status --short --branch` e lo SHA HEAD.
2. Conferma di essere su `refactor/src-domain-layout`; non passare a `main`.
3. Registra qui il checkpoint di partenza e controlla i file coinvolti prima di modificarli.

Dopo ogni gruppo:

1. Verifica il diff e lo stato Git; accertati che i cambiamenti siano limitati al gruppo previsto.
2. Esegui le verifiche autorizzate e registra risultati effettivi, inclusi gli esiti negativi e ciò che non è stato eseguito.
3. Aggiorna le sezioni “Completate e verificate”, “Scritte ma non verificate”, “Incomplete”, test, errori noti e prossimo passo.
4. Registra lo SHA dell'ultimo checkpoint committato e l'eventuale diff non committato. Non creare commit senza l'autorizzazione richiesta.

Dopo un'interruzione, rileggi `AGENTS.md`, la roadmap, `.github/workflows/refactoring.md` e questo registro. Verifica branch, working tree e SHA; riprendi dal prossimo passo preciso documentato. Ispeziona ogni modifica scritta ma non verificata prima di considerarla completata. Non ripetere né dichiarare concluso un lavoro senza evidenza.

## Errori e rischi conosciuti

- Spostare moduli può cambiare la risoluzione di `use super::*`, `#[path]`, `include_str!` e import relativi; audit richiesto per ogni gruppo.
- I re-export pubblici e `pub(crate)` del dominio combat devono restare disponibili ai chiamanti esistenti.

## Prossima attività autorizzata

I Gruppi 1–6 sono COMPLETATI STRUTTURALMENTE E SOTTOPOSTI AD AUDIT STATICO. Il Gruppo 7 è autorizzato e in fase di preparazione. Formattazione, compilazione, suite dei test e GitHub Actions restano da eseguire; nessuna verifica tecnica finale è superata sullo SHA corrente. La verifica statica o `git diff --check` non sostituisce tali controlli.

## Criterio di modularizzazione aggiornato – 2026-11-08

- L'utente ha approvato un criterio valido per tutti i domini: valutare moduli Rust e test in base alle responsabilità e alla coesione, anche molto sotto le 1.000 righe. La soglia di 1.000 righe è un limite di attenzione, non il criterio minimo per una suddivisione.
- Suddividere quando le responsabilità sono distinguibili e il confine migliora concretamente l'architettura. Per i test, raggruppare funzionalità/comportamenti simili, separare responsabilità diverse, centralizzare fixture/helper condivisi ed evitare duplicazioni.
- Evitare file minuscoli e frammentazioni senza un vantaggio concreto. Oltre 1.000 righe esaminare sempre il file; oltre 1.500 righe documentare la motivazione e ottenere l'approvazione prevista.
- Il criterio si applica ai gruppi ancora da completare e all'audit complessivo di tutti i domini, compresi quelli già toccati dai Gruppi 2 e 3. Questa registrazione non autorizza né avvia il Gruppo 4 e non riapre modifiche Rust in questa attività.
- Modifiche di questa attività: solo `AGENTS.md`, `.github/workflows/refactoring.md`, `.github/workflows/benilla-classic-pipeline-revised-v3.md` e `CODEX_PROGRESS.md`. Nessun file Rust modificato. Nessun commit o operazione remota eseguiti.
- Verifica documentale: `git diff --check` superato senza errori; Git ha segnalato solo la conversione LF/CRLF già configurata per `AGENTS.md` e `CODEX_PROGRESS.md`. Nessun test, build o controllo Rust eseguito.
- Checkpoint prima delle modifiche: branch `refactor/src-domain-layout`, HEAD committato `a4bee565e306f630fbca5688a23cf891f658508c`; working tree già contenente esclusivamente i cambiamenti documentati dei Gruppi 2 e 3 in `src/`, `src/main.rs`, `CODEX_PROGRESS.md` e `.github/workflows/refactoring.md`. Nessuna modifica ulteriore inattesa rilevata.

## Gruppo 4 – Suddivisione per responsabilità: COMPLETATO STRUTTURALMENTE E VERIFICATO STATICAMENTE

Checkpoint di avvio del 2026-11-08:

- Branch: `refactor/src-domain-layout`; HEAD committato: `a4bee565e306f630fbca5688a23cf891f658508c`.
- Stato Git: le modifiche non committate dei Gruppi 2 e 3 e gli aggiornamenti documentali erano presenti e coerenti con il registro; nessuna modifica ulteriore inattesa. Sono preservate integralmente.
- Criterio: valutare sorgenti e test in tutti i domini per responsabilità separabili a qualunque dimensione; mantenere fixture/helper condivisi, evitare duplicazioni e frammentazioni artificiali. 1.000 righe è una soglia di attenzione.
- Perimetro: suddividere file e gruppi di test in base alla mappa operativa e alle responsabilità reali; non eseguire il riordino di domini assegnato al Gruppo 5.
- Verifiche consentite: audit statici e `git diff --check`. Nessun controllo Cargo, test o installazione di strumenti Rust.
- Questo era il checkpoint prima delle prime modifiche Rust del Gruppo 4; i batch successivi e l'audit finale sono registrati qui sotto.
- Revisione statica iniziale: censiti 104 file Rust nel checkout attuale. Le suite di test individuate con responsabilità distinte riguardano camera, combat events, summon, runtime aura, dispatcher, enemy AI/attacks, player attacks e content loader. In particolare `combat/summon/mod.rs` comprende implementazione e test unitari inline e supera 1.000 righe fisiche; `combat/core/events.rs` si avvicina alla soglia e contiene gruppi separabili di validazione, coda e sottoscrizioni.
- Prima di spostare codice è stata aggiornata la mappa operativa per aggiungere le destinazioni di test per responsabilità e una checklist del Gruppo 4. Il Gruppo 5 (riordino degli altri domini) resta fuori ambito.
- Suddivisioni pianificate: camera controlli/follow/visibilità; combat events contesto/coda/sottoscrizioni; summon tipi-spawn-runtime; aura operazioni-lifecycle/reactive; dispatcher subscriber/runtime reattivo; enemy AI acquisizione/lifecycle; enemy attacks idoneità/risoluzione danno; player attacks idoneità/risoluzione; content loader serializzazione/registrazione/validazione. Fixture e helper condivisi resteranno nei moduli parent.
- Batch 1 completato: i 15 test inline di `src/camera/mod.rs` sono stati estratti in `src/camera/tests/` e raggruppati per controlli/distanza, follow e visibilità. Conteggio statico dei nomi: 15 prima e 15 dopo; nessuna fixture duplicata. Restano le altre suite del perimetro.
- Batch 2 completato e verificato staticamente: `combat/core/events.rs` è stato convertito in `combat/core/events/mod.rs` con i test in `events/tests/` (validazione contesto, coda, sottoscrizioni); i 18 test inline di `combat/summon/mod.rs` sono ora in `summon/unit_tests/` (tipi/lifecycle, spawn, runtime effetti), senza interferire con i test già presenti in `summon/tests/`; i 10 test inline del loader sono in `content/loader/tests/` (serializzazione, registrazione, validazione). Il conteggio dei test estratti nel lotto è 33 + 18 + 10 = 61 prima e 61 dopo; fixture condivise centralizzate nei rispettivi parent. Durante l'estrazione degli eventi è stato rilevato e corretto un collocamento intermedio errato: l'implementazione è stata conservata in `events/mod.rs` e le fixture comuni riprese dal sorgente Git originale; nessun test è stato perso. Controllati dichiarazioni `mod` e percorsi; `git diff --check` superato. Nessun test/build o controllo Rust eseguito.
- Batch 3 completato e verificato staticamente: i 19 test del dispatcher sono in `subscriber_tests.rs` e `reactive_runtime_tests.rs`; i 5 test già esistenti di `area_tests.rs` restano inclusi. I 10 test runtime aura sono in `lifecycle_tests.rs`, `operation_tests.rs` e `reactive_tests.rs`; gli helper usati dai test reattivi sono stati centralizzati in `aura_tests/mod.rs`. Conteggio statico: 19 test dispatcher + 5 area invariati e 10 test aura. Dichiarazioni `mod` e il riferimento esterno a `aura_tests` verificati; `git diff --check` superato. Nessun test/build o controllo Rust eseguito.
- Batch 4 completato e verificato staticamente: 22 test enemy AI separati tra acquisizione bersagli e aggro/lifecycle; 17 test enemy attacks tra eleggibilità e risoluzione danno/cooldown; 15 test player attacks tra eleggibilità al cast e risoluzione abilità/danno/threat. Fixture comuni restano nei parent.
- Audit finale del Gruppo 4: inventario confrontato con la baseline acquisita prima delle modifiche: 159 nomi iniziali, 159 nomi attuali, 159 unici; nessun nome mancante, aggiunto o duplicato. Moduli dichiarati nelle nuove suite verificati staticamente e relativi file presenti. Riferimenti relativi `super::super` ricontrollati dopo l'aggiunta di un livello: aggiornati soltanto i path interessati nelle fixture di combat events e nei test di validazione del loader. Le suite `area_tests` restano incluse; helper e fixture condivisi rimangono nei parent. Nessun `#[path]`, `include_str!` o `include_bytes!` nei file ricollocati; i riferimenti `include_str!` nelle fixture summon già esistenti non sono stati spostati.
- Dimensioni dopo le suddivisioni: nessun file Rust sotto `src/` supera 1.000 righe.
- `git diff --check`: superato; restano solo gli avvisi Git di conversione LF/CRLF preesistenti su `AGENTS.md` e `CODEX_PROGRESS.md`.
- Modifiche strutturali: test di camera, combat events, summon unitari, content loader, runtime aura, dispatcher, enemy AI, enemy attacks e player attacks suddivisi secondo la mappa; `combat/core/events.rs` convertito in `combat/core/events/mod.rs`. Nessuna modifica intenzionale a meccaniche o comportamento; correzioni ai riferimenti relativi sono esclusivamente strutturali.
- Test eseguiti: nessuno. Formattazione Rust, compilazione e suite di test non eseguite per istruzione dell'utente; restano rinviate alla verifica finale tramite GitHub Actions. Nessun tentativo di installazione o esecuzione Cargo.
- Errori conosciuti: nessun errore statico residuo rilevato. La verifica di compilazione e dei test runtime è ancora pendente.
- Stato Git checkpoint finale: branch `refactor/src-domain-layout`; HEAD committato invariato `a4bee565e306f630fbca5688a23cf891f658508c`; working tree contiene le modifiche non committate già presenti dei Gruppi 2 e 3, gli aggiornamenti documentali precedenti e le modifiche del Gruppo 4. Nessuna modifica è stata scartata. Nessun commit, push, upload, merge o Pull Request eseguito. Gruppo 5 non iniziato.
- Prossima attività: fermarsi e attendere l'autorizzazione dell'utente; non iniziare il Gruppo 5.

## Gruppo 5 – Riordino degli altri domini: COMPLETATO STRUTTURALMENTE E VERIFICATO STATICAMENTE

Checkpoint di avvio del 2026-11-08, prima di qualsiasi modifica del Gruppo 5:

- Branch verificato: `refactor/src-domain-layout`.
- HEAD committato verificato: `a4bee565e306f630fbca5688a23cf891f658508c`.
- Working tree: modifiche locali non committate dei Gruppi 2–4 e aggiornamenti documentali già registrati; staging vuoto. Il quadro corrisponde al checkpoint finale del Gruppo 4, senza ulteriori modifiche inattese. Tutte le modifiche preesistenti sono preservate.
- Documenti riletti integralmente: `AGENTS.md`, roadmap v3, `.github/workflows/refactoring.md` e questo registro.
- Perimetro autorizzato: Content, Character, Player, Navigation, Fellowship e Interaction; sono ammesse modifiche ad altri domini soltanto per collegamenti modulari necessari.
- Criterio: responsabilità e coesione a qualunque dimensione; test per comportamento; fixture/helper condivisi; evitare duplicazioni e frammentazione. Solo cambiamenti strutturali.
- Stato al checkpoint: nessun file del Gruppo 5 modificato in questa attività. Non sono stati eseguiti test/build/Cargo; nessun commit o operazione remota.
- Audit iniziale della mappa: `content/database.rs` e `content/database/tests.rs` sono ancora wrapper nelle posizioni legacy, mentre la mappa assegna loro `content/database/mod.rs` e `content/database/tests/mod.rs`. I moduli database lookup/registration/validation e i test aura, Classic, reference e registration sono già separati.
- Decisione strutturale: mantenere le fixture condivise nel parent `database/tests/mod.rs` e isolare i tre test inline di validazione talenti in `talent_validation_tests.rs`; il test singolo sull'ordine delle dipendenze resta nel parent per evitare un file di un solo test. La mappa è stata aggiornata prima degli spostamenti.
- Audit iniziale degli altri domini: Character è già diviso in build/class/race e i suoi quattro test formano una suite coerente sulla matrice razza-classe/fazioni; Player separa input, movimento e helper; Navigation separa navmesh, movement/helper e targeting; Fellowship raggruppa un unico sottosistema di membership; Interaction è un unico flusso pointer-hover/click. Non si pianificano ulteriori frammentazioni di questi moduli.
- Duplicazione rilevata: `player/movement_helpers.rs` e `navigation/enemy/movement_helpers.rs` hanno algoritmi e costanti equivalenti per collisione, discesa, caduta, step-up e scorrimento su superfici. La mappa è stata aggiornata prima del codice per introdurre `src/movement/mod.rs` come implementazione comune, mantenendo facade/alias specifici e le due serie di test esistenti. `JUMP_SPEED` resta specifico del player.
- Batch 1 Content avviato dopo ricontrollo branch/HEAD/stato: conversione dei due wrapper database alla struttura già mappata e separazione dei tre test di validazione talenti; nessuna altra parte dei domini autorizzati sarà modificata in questo batch.
- Batch 1 Content completato e verificato staticamente: `content/database.rs` è ora `content/database/mod.rs`; `content/database/tests.rs` è ora `content/database/tests/mod.rs`; i tre test talenti sono in `tests/talent_validation_tests.rs` e il test sulle dipendenze resta nel parent. Tutte le fixture condivise sono rimaste nel parent. I quattro nomi di test preesistenti sono stati ricollocati senza aggiunte o rimozioni; le dichiarazioni `mod` puntano ai percorsi attesi. `git diff --check` superato (solo gli avvisi LF/CRLF sui documenti preesistenti). Nessun test o build eseguito.
- Audit Content: definizioni per tipo, database (lookup/registration/validation), loader e suite aura/Classic/reference/registration sono già organizzati per responsabilità; nessun ulteriore split del codice Content è giustificato da questa ispezione.
- Batch 2 Player/Navigation completato: creato `src/movement/mod.rs` con l'implementazione interna condivisa; le facade `src/player/movement_helpers.rs` e `src/navigation/enemy/movement_helpers.rs` re-esportano i simboli con gli alias specifici precedenti. `JUMP_SPEED` resta specifico del player. Le tre prove inline in ciascuna facade sono state preservate.
- Audit statico finale: multinsieme dei nomi dei test confrontato con la baseline pre-Gruppo 5: 149 occorrenze e 146 nomi distinti prima e dopo; nessun test mancante, aggiunto o duplicato rispetto alla baseline. Fixture condivise restano nei parent; i due `include_str!` preesistenti delle fixture Classic sono stati ricontrollati dopo la conversione del wrapper e i percorsi risultano ancora coerenti. Nessun `#[path]` o `include_bytes!` nei domini interessati.
- Dichiarazioni `mod` e relativi percorsi controllati nei sei domini; `mod movement;` in `main.rs` punta a `src/movement/mod.rs`. I vecchi `src/content/database.rs` e `src/content/database/tests.rs` non esistono più; le destinazioni `database/mod.rs` e `database/tests/mod.rs` e il nuovo `talent_validation_tests.rs` esistono. Il wiring dei plugin non è stato alterato in questo gruppo; il diff di `main.rs` aggiunge solo il modulo privato condiviso e mantiene l'import dei plugin da `navigation` già introdotto nel Gruppo 2.
- Fellowship e Interaction sono stati ispezionati e mantenuti coesi, senza frammentazioni; Character e gli altri moduli già separati non richiedevano ulteriori split. Nessuna modifica intenzionale a comportamento, formule, meccaniche, dati o API pubbliche.
- `git diff --check`: superato; Git segnala soltanto avvisi di conversione LF/CRLF per file con formattazione preesistente. Audit di `#[path]`, `include_str!` e `include_bytes!`: due `include_str!` Classic presenti e verificati, nessun altro riferimento nei domini Group5.
- Formattazione, compilazione e suite di test: NON ESEGUITE, come richiesto; restano rinviate alla verifica finale tramite GitHub Actions. Nessun comando Cargo o installazione strumenti eseguito.
- File del Gruppo 5: spostati `src/content/database.rs` → `src/content/database/mod.rs` e `src/content/database/tests.rs` → `src/content/database/tests/mod.rs`; creati `src/content/database/tests/talent_validation_tests.rs` e `src/movement/mod.rs`; modificati `src/content/database/mod.rs`, `src/content/database/tests/mod.rs`, `src/main.rs`, `src/player/movement_helpers.rs`, `src/navigation/enemy/movement_helpers.rs`, `.github/workflows/refactoring.md` e questo registro. Nessun file Rust di Fellowship o Interaction modificato. Le altre modifiche preesistenti dei Gruppi 2–4 e documentali sono state preservate.
- Errori noti/incomplete: nessun problema statico residuo osservato. Le verifiche tecniche Rust restano pendenti; il registro non le dichiara superate.
- Checkpoint finale: branch `refactor/src-domain-layout`, HEAD committato invariato `a4bee565e306f630fbca5688a23cf891f658508c`; working tree contiene modifiche non committate dei Gruppi 2–5 e documenti correlati; staging vuoto. `git diff --check` superato. Nessun commit, push, upload, merge o PR eseguito.
- Prossimo passo: fermarsi e attendere nuove istruzioni. Gruppo 6 non iniziato.

## Gruppo 6 – Audit strutturale complessivo: COMPLETATO E VERIFICATO STATICAMENTE

Checkpoint di avvio del 2026-11-08, prima dell'audit completo:

- Branch verificato: `refactor/src-domain-layout`.
- HEAD verificato: `a4bee565e306f630fbca5688a23cf891f658508c`.
- Stato Git iniziale: working tree modificato dalle attività intenzionali dei Gruppi 2–5 e dai documenti precedenti; staging vuoto; branch avanti di due commit rispetto a `origin/refactor/src-domain-layout`. Nessuna modifica inattesa rispetto al checkpoint finale del Gruppo 5.
- Riletti integralmente `AGENTS.md`, roadmap v3, `.github/workflows/refactoring.md` e questo registro.
- Attività autorizzata: audit completo di tutti i sorgenti sotto `src/`, con eventuali ulteriori suddivisioni strutturali utili secondo il criterio di responsabilità aggiornato. Preservare tutte le modifiche esistenti.
- Inventario iniziale di questo audit: 133 file Rust sotto `src/`; conteggio statico iniziale dei test: 660 occorrenze e 656 nomi distinti. Il dato va confrontato con la baseline storica (660 test Run #522) e con gli inventari registrati nei Gruppi 3–5, tenendo conto delle sovrapposizioni tra gruppi.
- Dimensione iniziale del Gruppo 6: conteggio fisico tramite `ReadAllLines`; il file più lungo è `src/combat/damage/mitigation.rs` con 893 righe. Nessun file supera 1.000 o 1.500 righe.
- Prima verifica della mappa: 133 file `.rs` correnti; tutte le destinazioni del registro risultano presenti tranne i sei aggregatori combat aggiunti, ora documentati in una tabella supplementare. `src/movement/mod.rs` era menzionato due volte come destinazione condivisa; la mappa ora lo registra una sola volta e rimanda alla voce comune.
- Suddivisione eseguita nella suite di targeting: i 13 casi in `combat/targeting/selection/tests/mod.rs` sono separati in `tab_tests.rs` (7 casi) e `click_tests.rs` (6 casi); le fixture comuni restano in `tests/mod.rs`.
- Nessun test, build o controllo Cargo eseguito. Nessun commit o aggiornamento remoto.
- Audit statico finale: le 88 sorgenti baseline sono tutte mappate in 129 destinazioni operative; aggiungendo i 6 aggregatori combat generati si ottengono 135 file Rust, coincidenti con il checkout. Zero dichiarazioni `mod` senza destinazione e zero macro/percorso con target mancante. Test e re-export confrontati con HEAD senza differenze.
- Inventario dei test: 660 occorrenze e 656 nomi distinti sia in HEAD sia nel checkout; multinsieme identico. API di re-export confrontate: 67 voci combat e 49 Content, senza differenze.
- Ordine: sequenza dei 14 sistemi di `CombatPlugin` identica a HEAD; ordine di registrazione plugin in `main.rs` invariato. Nessun riferimento ai vecchi percorsi di moduli sotto `src/`.
- Errori statici: nessuno rilevato. Limite noto: senza compilazione non è possibile confermare la risoluzione completa dei nomi Rust e delle visibilità né l'esecuzione dei test.
- File Group 6: modificato `src/combat/targeting/selection/tests/mod.rs`, creati `src/combat/targeting/selection/tests/tab_tests.rs` e `click_tests.rs`; aggiornate la mappa e le checklist in `.github/workflows/refactoring.md` e questo registro. Nessun sorgente di produzione modificato nel Gruppo 6.
- `git diff --check`: superato; Git segnala soltanto avvisi LF/CRLF su file preesistenti.
- Stato finale Gruppo 6: branch `refactor/src-domain-layout`, HEAD `a4bee565e306f630fbca5688a23cf891f658508c`, staging vuoto, working tree con modifiche non committate dei Gruppi 2–6 e documenti correlati, branch avanti di due commit rispetto a origin.
- Non eseguiti: `cargo fmt`, build, test, GitHub Actions o installazioni. Sono rinviati al Gruppo 7, che non è iniziato.
- Prossimo passo preciso: fermarsi e attendere ulteriori istruzioni; non iniziare il Gruppo 7.

## Gruppo 3 – Riordino del dominio combat: COMPLETATO E VERIFICATO STATICAMENTE

Checkpoint finale del 2026-11-08:

- Branch: `refactor/src-domain-layout`.
- HEAD committato: `a4bee565e306f630fbca5688a23cf891f658508c`.
- HEAD committato invariato: `a4bee565e306f630fbca5688a23cf891f658508c`; il lavoro dei Gruppi 2 e 3 è presente nel working tree non committato.
- Struttura eseguita: moduli combat ricollocati in `aura`, `core`, `damage`, `death`, `displacement`, `effects`, `enemy`, `player`, `summon` e `targeting`; creati i sei aggregatori previsti (`aura/mod.rs`, `core/mod.rs`, `effects/mod.rs`, `enemy/mod.rs`, `player/mod.rs`, `targeting/mod.rs`). Import, dichiarazioni `mod` e re-export aggiornati.
- Suddivisioni eseguite secondo la mappa: test aura in stacking/periodic/lifecycle-validation; test inline di effect execution estratti in `effects/execution/tests/`; test effect runtime in queue/execution; test summon in spawn/combat/lifecycle/classic-pack con fixture condivise.
- Audit statico: la mappatura combat è stata confrontata con l'albero presente; tutte le destinazioni previste esistono e non risultano file alle vecchie posizioni. I 40 file trasferiti senza modifiche hanno mantenuto il blob Git originale. Controllate le dichiarazioni `mod` e i riferimenti `#[path]`, `include_str!` e `include_bytes!`; tutti i target verificati esistono. Nessun riferimento obsoleto ai vecchi percorsi combat è rimasto.
- Test preservati: inventario statico delle funzioni di test pari a 496 prima e 496 dopo; nessun test rimosso o ridotto. I 67 re-export pubblici confrontati corrispondono alla baseline. L'ordine di registrazione dei sistemi in `combat` è invariato. Nessuna modifica intenzionale a logica o comportamento.
- Dimensioni: 63 file `.rs` sotto `src/combat/`, nessuno oltre 1.000 righe.
- `git diff --check`: superato, senza errori; Git ha emesso soltanto un avviso di conversione LF/CRLF su `CODEX_PROGRESS.md`.
- Formattazione Rust, compilazione e test: NON ESEGUITI, come richiesto; rinviati alla verifica finale tramite GitHub Actions. Non sono stati effettuati ulteriori tentativi o installazioni di strumenti Rust.
- Modifiche non committate comprendono anche il Gruppo 2 già autorizzato e documentato; non è stato creato alcun commit né eseguito push, upload, merge o Pull Request.
- Prossimo passo: fermarsi e attendere istruzioni dell'utente; il Gruppo 4 non è iniziato.

## Gruppo 7 – Preparazione della verifica tecnica finale: IN PREPARAZIONE

Checkpoint di avvio del 2026-11-08:

- Branch verificato: `refactor/src-domain-layout`.
- HEAD verificato: `a4bee565e306f630fbca5688a23cf891f658508c`.
- Stato iniziale: working tree modificato dalle attività documentali e strutturali già registrate dei Gruppi 2–6; staging vuoto; branch avanti di due commit rispetto a `origin/refactor/src-domain-layout`. Nessuna modifica preesistente è stata ripristinata o scartata.
- Riletti `AGENTS.md`, roadmap v3, `.github/workflows/refactoring.md`, questo registro e `.github/workflows/build.yml`.
- Gruppi 1–6: completati strutturalmente e sottoposti ad audit statico secondo i registri precedenti.
- Stato tecnico: formattazione, compilazione, suite completa dei test e GitHub Actions NON ESEGUITE sullo SHA corrente. Nessuna verifica tecnica finale è dichiarata superata.

### Analisi del workflow CI

- Prima della correzione, `.github/workflows/build.yml` eseguiva `cargo fmt --all -- --check` dentro un blocco condizionale. Al fallimento impostava `status=warning`, emetteva un warning e continuava senza restituire codice di errore; per questo la formattazione non bloccava il workflow.
- Correzione autorizzata applicata: il controllo ora è il passo diretto `run: cargo fmt --all -- --check`. Il riepilogo del job riporta la formattazione come superata solo dopo il successo del job; il fallimento del comando termina il passo e impedisce al workflow di risultare superato.
- Nessun altro controllo o job è stato modificato: matrice Linux/Windows, compilazione, test, avvio headless e caricamento artifact sono preservati.
- Il workflow già prevede `cargo check --all-targets`, `cargo build --release`, `cargo test --release` e `cargo run --release` con `CI=true` sui job Linux e Windows. I trigger sono push solo su `main`, Pull Request diretta a `main` e `workflow_dispatch` manuale. La verifica sul branch di lavoro non è stata avviata.
- I job prevedono il caricamento di artifact Windows e Linux. Nessun workflow o caricamento è stato avviato.

### Modifiche documentali di preparazione

- Aggiornati roadmap e piano operativo per indicare i Gruppi 1–6 completati strutturalmente e sottoposti ad audit statico, Gruppo 7 autorizzato/in preparazione e verifiche tecniche ancora pendenti.
- Nella precedente preparazione era stato registrato che il controllo rustfmt in `build.yml` era non bloccante e che la correzione era in attesa di autorizzazione. L'autorizzazione successiva ha reso il controllo bloccante; il successivo allineamento documentale è registrato nella sezione finale di questo file.
- File modificati in questa attività: `.github/workflows/build.yml` e `CODEX_PROGRESS.md`. I tre documenti aggiornati nella precedente preparazione e tutte le modifiche Rust preesistenti sono stati preservati.
- File non modificati in questa attività: `AGENTS.md`, roadmap v3, `.github/workflows/refactoring.md` e tutti i file Rust.
- Verifiche: diff completo di `build.yml` e `CODEX_PROGRESS.md` ispezionato; `git diff --check` superato senza errori (solo avvisi LF/CRLF). Controllo statico della struttura YAML: passo diretto e indentazione sotto `steps` verificati, nessuna tabulazione, vecchio blocco condizionale assente, job Linux/Windows e passi build/test/headless/artifact presenti. Non è disponibile un parser YAML locale già installato; nessuno strumento è stato installato.
- Nessun test, build, comando Cargo, GitHub Actions, commit o operazione remota eseguiti.

### Stato delle modifiche e prossimo passo

- Modifiche completate e verificate: Gruppi 1–6 come riportato nei rispettivi checkpoint statici. Correzione del workflow e aggiornamento del registro verificati tramite diff e controllo statico; `git diff --check` superato.
- Modifiche incomplete: nessuna modifica funzionale o Rust autorizzata; la verifica tecnica finale resta pendente.
- Test superati/falliti: nessun test è stato eseguito in questo gruppo.
- Test, formattazione e compilazione non eseguiti: tutti, rinviati alla verifica tecnica finale.
- Errori noti: non è stato possibile usare un parser YAML dedicato; il controllo è statico/manuale e non equivale a una validazione tramite parser o a una run GitHub Actions. Il workflow non è stato eseguito.
- Prossimo passaggio preciso: fermarsi e attendere ulteriori istruzioni. Il workflow non è stato eseguito; formattazione Rust, compilazione, test e GitHub Actions restano pendenti.
- Ultimo checkpoint Git committato: `a4bee565e306f630fbca5688a23cf891f658508c`.

## Allineamento documentale del controllo CI bloccante

- Checkpoint verificato: branch `refactor/src-domain-layout`, HEAD `a4bee565e306f630fbca5688a23cf891f658508c`; working tree già modificato dalle attività precedenti e staging vuoto. Tutte le modifiche preesistenti sono state preservate.
- Rilette integralmente roadmap v3, `.github/workflows/refactoring.md` e questo registro prima dell'allineamento.
- Aggiornati roadmap e piano operativo per indicare che `cargo fmt --all -- --check` è ora bloccante in `build.yml`.
- Stato tecnico invariato: il workflow non è stato eseguito; formattazione, compilazione e test sullo SHA finale restano da verificare. Nessun risultato CI è dichiarato superato.
- Verificata la coerenza delle dichiarazioni sul controllo bloccante e sulle verifiche ancora pendenti nei tre documenti. `git diff --check` superato senza errori; Git ha emesso soltanto avvisi di conversione LF/CRLF su file preesistenti.
- File modificati in questo allineamento: roadmap v3, `.github/workflows/refactoring.md` e `CODEX_PROGRESS.md`. `build.yml`, file Rust e altre modifiche esistenti non sono stati modificati in questo passaggio.
- Nessun test, build, workflow, commit o operazione remota eseguiti.
- Prossimo passaggio: fermarsi e attendere ulteriori istruzioni; non avviare workflow o verifiche tecniche senza autorizzazione.
