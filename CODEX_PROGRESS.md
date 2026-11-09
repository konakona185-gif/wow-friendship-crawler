# Registro di avanzamento Codex

## 1. Stato operativo corrente

**Ultimo aggiornamento: 2026-11-09**

- Progetto: `WoW-Friendship-Crawler` (`konakona185-gif/wow-friendship-crawler`).
- Branch di lavoro: `refactor/src-domain-layout`.
- Ultimo HEAD remoto verificato: `1a3c5f4d24fd25d599596b1d16a7fc4006d0106b`.
- Ultimo commit: `Update lifecycle.rs`.
- `main` non è stato modificato dal lavoro di refactoring.
- Fase strutturale 3.5: Gruppi 1–6 completati e sottoposti ad audit statico; Gruppo 7 completato con CI finale verde.
- Run finale: GitHub Actions #544, successo su Linux e Windows.
- Fase 4: non ancora avviata; resta subordinata alla preparazione e all'integrazione del branch e all'autorizzazione dell'utente.

### Ultimo stato della verifica CI

La verifica tecnica finale è stata completata con la Run #544.

- Branch: `refactor/src-domain-layout`.
- SHA verificato: `1a3c5f4d24fd25d599596b1d16a7fc4006d0106b`.
- Run ID: `42587661206`.
- URL: https://github.com/konakona185-gif/wow-friendship-crawler/actions/runs/42587661206
- Evento: `workflow_dispatch`.
- Esito: `success`.

Controllo preliminare Rust:
- `cargo fmt --all -- --check`: superato.
- Installazione delle dipendenze Linux: superata.
- `cargo check --all-targets`: superato.
- Riepilogo del controllo preliminare: superato.

Compilazione e test Linux:
- Compilazione Linux: superata.
- Test unitari Linux: superati.
- Test headless Linux: superati.
- Artifact `gioco-linux`: caricato correttamente.

Compilazione e test Windows:
- Compilazione Windows: superata.
- Test unitari Windows: superati.
- Test headless Windows: superati.
- Artifact `gioco-windows`: caricato correttamente.

La verifica degli artifact ha confermato la presenza di entrambi:
- `gioco-linux`;
- `gioco-windows`.

La Run #544 costituisce la verifica tecnica completa della struttura riorganizzata sullo SHA indicato. Non viene attribuito a questa run un numero di test eseguiti diverso da quello effettivamente riportato nei log; il riferimento storico della baseline resta la Run #522 con 660 test per piattaforma.

### Errori di compilazione corretti prima della verifica finale

Durante la verifica sono emersi due errori di visibilità relativi alla funzione `summon_lifecycle_system`:

- E0364: re-export non consentito per una funzione con visibilità insufficiente.
- E0603: funzione non accessibile dal modulo chiamante.

La causa era la visibilità `pub(super)` della funzione in `src/combat/summon/lifecycle.rs`.

La correzione applicata ha impostato la visibilità:

`pub(in crate::combat)`

La funzione è quindi accessibile all'interno del dominio Combat senza essere esposta pubblicamente all'intero progetto.

La correzione è stata inclusa nel commit verificato dalla Run #544:

`1a3c5f4d24fd25d599596b1d16a7fc4006d0106b`

### Stato della Fase 3.5

I Gruppi 1–6 sono stati completati e sottoposti ad audit statico. Il Gruppo 7 è stato completato con la Run #544 interamente verde.

La verifica finale ha coperto:
- formattazione;
- controllo statico di tutti i target Rust;
- compilazione;
- test unitari;
- test headless;
- piattaforma Linux;
- piattaforma Windows;
- produzione e caricamento degli artifact.

La Fase 3.5 può quindi essere considerata tecnicamente completata.

Rimane da concludere la procedura di integrazione del branch `refactor/src-domain-layout` in `main`, preservando i tre commit documentali successivi presenti su `main`.

### Situazione dei branch prima dell'integrazione

Confronto verificato tra `main` e `refactor/src-domain-layout`:

- `refactor/src-domain-layout`: 33 commit avanti rispetto a `main`.
- `refactor/src-domain-layout`: 3 commit indietro rispetto a `main`.
- Stato del confronto: `diverged`.
- HEAD di `main` verificato: `4c48fcca9358402426d2508d5e645b40e48f85db`.
- Merge base: `454874a358a18e7e565043d797e3c42ddc46323f`.

I tre commit successivi di `main` riguardano la gestione documentale:
- eliminazione del vecchio `.github/workflows/benilla-classic-pipeline-revised-v2.md`;
- eliminazione di `.github/workflows/refactoring.md`;
- aggiunta della versione V3 della roadmap e della versione aggiornata di `refactoring.md`.

La versione presente sul branch di refactoring contiene successivi aggiornamenti e informazioni operative da preservare. Prima dell'integrazione occorre verificare e riconciliare questi documenti, senza perdere contenuti o ripristinare versioni obsolete.

La Pull Request dovrà essere preparata da:

`refactor/src-domain-layout`

verso:

`main`

Nessun merge è stato eseguito e `main` non è stato modificato durante il refactoring.

### Prossimo passaggio

1. Aggiornare questo registro, la roadmap principale e il documento tecnico di refactoring sul branch `refactor/src-domain-layout`.
2. Verificare che i tre documenti siano coerenti tra loro e con la Run #544.
3. Esaminare la differenza tra i branch e preservare i tre commit documentali successivi di `main`.
4. Preparare la Pull Request verso `main`.
5. Esaminare la Pull Request e le verifiche CI prima dell'integrazione.
6. Non iniziare la Fase 4 senza autorizzazione esplicita dell'utente.

---

## 2. Regole permanenti del repository

### Branch e sicurezza Git

- Lavorare esclusivamente sul branch `refactor/src-domain-layout` durante questa attività.
- Prima di ogni gruppo di modifiche verificare branch attivo, HEAD e stato del working tree.
- Non passare a `main` e non modificarlo direttamente.
- Non eseguire push, merge o creare/aggiornare Pull Request senza autorizzazione esplicita.
- Non creare commit senza la relativa autorizzazione.
- Preservare tutte le modifiche preesistenti; non ripristinare o scartare modifiche senza averne accertato origine e scopo.
- Non considerare un aggiornamento completato finché il file non è stato verificato sul branch remoto.

### Modalità di lavoro

- Per modifiche strutturali o multilinea fornire il file completo, non frammenti da ricomporre.
- Procedere un file alla volta e verificare ogni salvataggio prima di passare al successivo.
- Raggruppare le modifiche correlate prima di avviare i controlli CI, evitando esecuzioni ripetute per cambiamenti documentali minori.
- Non dichiarare superati controlli che non siano stati realmente eseguiti.
- Distinguere sempre audit statico, formattazione, compilazione, test e avvio headless.
- Conservare i risultati negativi e le verifiche mancanti nella documentazione.

### Documenti di continuità

- `AGENTS.md`: vincoli permanenti e istruzioni operative del repository.
- `.github/workflows/benilla-classic-pipeline-revised-v3.md`: roadmap principale del progetto e delle sue fasi.
- `CODEX_PROGRESS.md`: registro operativo e checkpoint per riprendere il lavoro dopo un'interruzione.
- `.github/workflows/refactoring.md`: piano tecnico dettagliato usato durante il refactoring. È utile per completare e verificare questa fase, ma non è necessario mantenerlo indefinitamente: dopo la CI finale verde, le informazioni ancora utili devono essere già trasferite nella roadmap e in questo registro. La sua eventuale eliminazione sarà effettuata separatamente e con autorizzazione dell'utente.

---

## 3. Obiettivo e vincoli del refactoring

La Fase 3.5 è una fase preparatoria obbligatoria prima della Fase 4.

Obiettivo: riorganizzare i sorgenti Rust sotto `src/` in directory per dominio e sottosistema, collocando implementazioni, moduli di supporto, test e fixture in strutture coerenti.

Il refactoring deve rimanere strutturale:
- preservare comportamento osservabile e meccaniche di gioco;
- preservare API, visibilità, import e re-export pubblici;
- conservare tutti i test e le fixture;
- preservare l'ordine di registrazione dei sistemi e dei plugin;
- correggere riferimenti di modulo e percorsi soltanto quando necessario per la nuova struttura;
- evitare frammentazioni artificiali e duplicazioni.

### Criterio di modularizzazione approvato

La suddivisione dei file non dipende esclusivamente dal numero di righe.

- Valutare responsabilità e coesione anche sotto le 1.000 righe.
- Separare responsabilità realmente distinte quando il confine migliora l'architettura.
- Organizzare i test per comportamento o funzionalità.
- Mantenere fixture e helper condivisi nei moduli parent o in un punto comune appropriato.
- Evitare file minuscoli e suddivisioni prive di un vantaggio concreto.
- Oltre 1.000 righe, esaminare sempre il file.
- Oltre 1.500 righe, documentare la motivazione e ottenere l'approvazione prevista.

---

## 4. Baseline e ricognizione iniziale

La preparazione iniziale è stata completata e verificata il 2026-11-08.

### Inventario iniziale

- 88 file Rust in 18 directory.
- Dimensione del checkout Windows: 948.467 byte.
- Dimensione dei blob Git della baseline: 913.840 byte.
- Differenza: 34.627 byte dovuti alla conversione delle terminazioni LF in CRLF con `core.autocrlf=true`.
- Riconciliazione: 913.840 + 34.627 = 948.467 byte.
- Tutti gli 88 file Rust sono stati confrontati con la baseline e risultavano identici prima degli spostamenti.

### Mappatura iniziale

- 88 percorsi sorgente.
- 98 destinazioni uniche previste.
- Nessun sorgente mancante o non mappato.
- Nessuna destinazione duplicata.

Le destinazioni aggiuntive derivavano dalla suddivisione pianificata di file di test di grandi dimensioni.

### Baseline storica dei test

La Run CI #522 aveva superato 660 test su Linux e Windows e verificato l'avvio headless allo SHA:

`55e001ec7dc06f5d9bbb3107f3b5ef3f96354b38`

Questo risultato è storico e non costituisce una verifica della struttura riorganizzata o dello SHA attuale.

---

## 5. Gruppi strutturali completati

### Gruppo 1 – Preparazione: COMPLETATO E VERIFICATO

- Ricognizione di `src/`, test, dimensioni e dipendenze.
- Inventario iniziale di 88 file Rust in 18 directory.
- Riconciliazione delle dimensioni Git/checkout Windows.
- Verifica della mappa iniziale: 88 sorgenti e 98 destinazioni uniche.
- Confronto dei contenuti con la baseline iniziale.
- Nessuna modifica ai sorgenti Rust in questa fase.

### Gruppo 2 – Moduli top-level ed enemy navigation: STRUTTURA COMPLETATA

Convertiti i moduli top-level in directory con `mod.rs`:

- `camera.rs`
- `character.rs`
- `combat.rs`
- `content.rs`
- `fellowship.rs`
- `interaction.rs`
- `navigation.rs`
- `player.rs`

Spostati anche:
- `character/tests.rs` in `character/tests/mod.rs`;
- `enemy_navigation.rs` e i relativi moduli figli sotto `navigation/enemy/`.

Aggiornato `main.rs` per importare `EnemyNavigationPlugin` dal dominio `navigation`.

Verifiche statiche:
- tutte le destinazioni previste presenti;
- nessun vecchio percorso residuo;
- hash dei 12 file trasferiti senza modifiche identici agli originali;
- dichiarazioni `mod` e percorsi verificati;
- nessun riferimento obsoleto al modulo top-level `enemy_navigation`;
- ordine di registrazione dei plugin invariato;
- test trasferiti preservati;
- `git diff --check` superato.

Formattazione Rust, compilazione e test non eseguiti in questa fase.

### Gruppo 3 – Riordino del dominio combat: STRUTTURA COMPLETATA

Riorganizzati i moduli combat in aree coerenti:

- aura;
- core;
- damage;
- death;
- displacement;
- effects;
- enemy;
- player;
- summon;
- targeting.

Creati i sei aggregatori previsti:
- `aura/mod.rs`;
- `core/mod.rs`;
- `effects/mod.rs`;
- `enemy/mod.rs`;
- `player/mod.rs`;
- `targeting/mod.rs`.

Suddivisioni principali:
- test aura per stacking, periodicità e lifecycle/validation;
- test di effect execution estratti in `effects/execution/tests/`;
- test effect runtime separati per queue ed execution;
- test summon raggruppati per spawn, combat, lifecycle e Classic pack.

Verifiche statiche:
- 40 file trasferiti senza modifiche con hash Git identico agli originali;
- 496 funzioni di test prima e dopo;
- nessun test rimosso o ridotto;
- 67 re-export pubblici combat invariati;
- ordine di registrazione dei sistemi invariato;
- riferimenti `#[path]`, `include_str!` e `include_bytes!` verificati;
- nessun file in `src/combat/` oltre 1.000 righe;
- `git diff --check` superato.

Formattazione Rust, compilazione e test non eseguiti in questa fase.

### Gruppo 4 – Suddivisione per responsabilità: STRUTTURA COMPLETATA

Estratti e raggruppati i test per responsabilità in:
- camera;
- combat events;
- summon;
- content loader;
- runtime aura;
- dispatcher;
- enemy AI;
- enemy attacks;
- player attacks.

Dettagli dei batch:
- camera: 15 test separati per controlli/distanza, follow e visibilità;
- combat events: test di validazione, coda e sottoscrizioni;
- summon: 18 test unitari separati in suite coerenti;
- content loader: 10 test separati per serializzazione, registrazione e validazione;
- dispatcher: 19 test separati, con i 5 test di `area_tests` preservati;
- runtime aura: 10 test suddivisi per lifecycle, operation e reactivity;
- enemy AI: 22 test;
- enemy attacks: 17 test;
- player attacks: 15 test.

Verifiche statiche:
- inventario dei nomi dei test: 159 prima e 159 dopo;
- nessun nome mancante, aggiunto o duplicato rispetto alla baseline del gruppo;
- fixture e helper condivisi mantenuti nei parent;
- moduli e percorsi verificati;
- riferimenti `super::super` aggiornati esclusivamente dove necessario dopo lo spostamento;
- nessun file Rust sotto `src/` oltre 1.000 righe;
- `git diff --check` superato.

Nessun test, build o controllo Cargo eseguito.

### Gruppo 5 – Riordino degli altri domini: STRUTTURA COMPLETATA

Dominio Content:
- `content/database.rs` convertito in `content/database/mod.rs`;
- `content/database/tests.rs` convertito in `content/database/tests/mod.rs`;
- tre test inline di validazione talenti spostati in `talent_validation_tests.rs`;
- fixture condivise mantenute nel modulo parent;
- test esistenti preservati.

Player e Navigation:
- creato `src/movement/mod.rs` per centralizzare l'implementazione condivisa degli helper di movimento;
- `player/movement_helpers.rs` e `navigation/enemy/movement_helpers.rs` mantenuti come facade con alias specifici;
- `JUMP_SPEED` resta specifico del player;
- test delle due facade preservati.

Character, Fellowship e Interaction sono stati ispezionati. Non sono state introdotte suddivisioni prive di un vantaggio architetturale concreto.

Verifiche statiche:
- 149 occorrenze di test e 146 nomi distinti prima e dopo;
- nessun test mancante, aggiunto o duplicato rispetto alla baseline;
- fixture e percorsi `include_str!` Classic verificati;
- dichiarazioni `mod` e percorsi dei sei domini controllati;
- `git diff --check` superato.

Nessun test, build o controllo Cargo eseguito.

### Gruppo 6 – Audit strutturale complessivo: COMPLETATO E VERIFICATO STATICAMENTE

Audit di tutti i sorgenti sotto `src/`, compresi i domini già riorganizzati.

Risultati:
- 88 sorgenti baseline mappati in 129 destinazioni operative;
- 6 aggregatori combat aggiuntivi;
- totale: 135 file Rust, coincidenti con il checkout;
- zero dichiarazioni `mod` senza destinazione;
- zero target mancanti nei riferimenti a percorsi e macro controllati;
- 660 occorrenze di test e 656 nomi distinti sia in HEAD sia nel checkout;
- multinsieme dei test identico;
- 67 re-export combat e 49 Content invariati;
- ordine dei 14 sistemi di `CombatPlugin` invariato;
- ordine di registrazione dei plugin in `main.rs` invariato;
- nessun riferimento residuo ai vecchi percorsi dei moduli;
- file Rust più lungo: 893 righe;
- `git diff --check` superato.

La suite di targeting è stata suddivisa in `tab_tests.rs` (7 test) e `click_tests.rs` (6 test), mantenendo le fixture comuni nel modulo parent.

Limite dell'audit: senza compilazione non è possibile confermare definitivamente la risoluzione dei nomi Rust, delle visibilità o il comportamento runtime. Tale limite è stato successivamente superato dalla verifica tecnica finale del Gruppo 7.

---

## 6. Gruppo 7 – Verifica tecnica finale

<ins>Stato: COMPLETATO E VERIFICATO CON CI INTERAMENTE VERDE.</ins>

### Workflow

File: `.github/workflows/build.yml`.

Il controllo di formattazione è stato modificato in precedenza: `cargo fmt --all -- --check` è eseguito come passo diretto e bloccante, non come warning.

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

Per verificare il branch di lavoro è quindi necessario avviare manualmente il workflow selezionando il branch corretto.

### Run #536 – Prima esecuzione

- Branch: `refactor/src-domain-layout`.
- SHA testato: `0cfcf2a61fff40500c7f9aac7ebec7c50cae7c77`.
- Run ID: `42504899025`.
- URL: https://github.com/konakona185-gif/wow-friendship-crawler/actions/runs/42504899025
- Esito: FALLITA durante `cargo fmt --all -- --check`.

Errore:
`expected item after attributes`

Percorso:
`src/combat/effects/dispatcher/tests/reactive_runtime_tests.rs`

La causa è stata una direttiva `#[cfg(test)]` rimasta orfana dopo che la dichiarazione `mod area_tests` era stata spostata nel modulo parent `tests/mod.rs`, già protetto da `#[cfg(test)]`.

I job successivi Linux e Windows non sono stati eseguiti perché dipendenti dal job preliminare fallito. Non sono stati verificati:
- `cargo check`;
- build release;
- test release;
- avvio headless;
- artifact.

### Correzione della Run #536

Commit:

`b69aefcf7efe844d8c9bf32227bf04ec6cfca2ba`

Messaggio:

`Remove unnecessary lines from reactive_runtime_tests.rs`

Modifica:
- rimossa esclusivamente la direttiva `#[cfg(test)]` orfana;
- preservati tutti gli 11 test del file;
- preservati i test sibling;
- nessuna modifica funzionale.

### Aggiornamenti documentali precedenti alla verifica finale

Roadmap principale:
- file: `.github/workflows/benilla-classic-pipeline-revised-v3.md`;
- commit: `62fac5fb90a2dcb414106753374ade67e5c7c9d2`.

Piano operativo di refactoring:
- file: `.github/workflows/refactoring.md`;
- commit: `f15b9b02d84a165b2a67eb074c1d1c168b17c821`;
- modifica registrata: aggiornamento dello stato della Fase 5, della Run #536 e della correzione pubblicata.

### Correzioni di compilazione e visibilità successive

La verifica di compilazione ha successivamente individuato due diagnostiche relative a `summon_lifecycle_system`:

- E0364, re-export di una funzione con visibilità insufficiente;
- E0603, accesso alla funzione non consentito dal modulo chiamante.

La correzione è stata applicata in:

`src/combat/summon/lifecycle.rs`

La visibilità della funzione è stata modificata da `pub(super)` a:

`pub(in crate::combat)`

La funzione rimane circoscritta al dominio Combat e può essere utilizzata dall'aggregatore previsto.

### Run #544 – Verifica finale

- Branch: `refactor/src-domain-layout`.
- SHA testato: `1a3c5f4d24fd25d599596b1d16a7fc4006d0106b`.
- Run ID: `42587661206`.
- URL: https://github.com/konakona185-gif/wow-friendship-crawler/actions/runs/42587661206
- Evento: `workflow_dispatch`.
- Esito: SUCCESS.

Controllo preliminare:
- formattazione Rust superata;
- installazione delle dipendenze Linux superata;
- controllo statico di tutti i target Rust superato;
- `cargo check --all-targets` superato.

Linux:
- compilazione superata;
- test unitari superati;
- test headless superati;
- artifact `gioco-linux` caricato.

Windows:
- compilazione superata;
- test unitari superati;
- test headless superati;
- artifact `gioco-windows` caricato.

Gli artifact sono stati verificati tramite GitHub Actions:
- `gioco-linux`, ID `13661140684`;
- `gioco-windows`, ID `13660802759`.

La Run #544 è la verifica conclusiva sullo SHA `1a3c5f4d24fd25d599596b1d16a7fc4006d0106b`. I controlli preliminari, la compilazione, le suite di test e l'avvio headless sono stati completati con esito positivo su entrambe le piattaforme.

### Criteri di completamento del Gruppo 7

- `cargo fmt --all -- --check`: SUPERATO.
- `cargo check --all-targets`: SUPERATO.
- Build Linux e Windows: SUPERATE.
- Test unitari Linux e Windows: SUPERATI.
- Avvio headless Linux e Windows: SUPERATO.
- Artifact Linux e Windows: CARICATI E VERIFICATI.
- Audit statico dei sorgenti e dei test: COMPLETATO.
- SHA esatto della verifica finale: REGISTRATO.

Il Gruppo 7 è completato. La Fase 3.5 è tecnicamente conclusa e non costituisce più un blocco per la preparazione della Fase 4.

---

## 7. Limitazioni tecniche note

- Cargo, rustfmt e gli strumenti Rust non sono disponibili nell'ambiente locale Windows utilizzato durante la preparazione.
- Non sono stati installati strumenti Rust né modificate configurazioni dell'ambiente.
- Le verifiche statiche non sostituiscono formattazione, compilazione o test; la Run #544 ha completato positivamente tali controlli sullo SHA verificato.
- La Run #536 è fallita prima dei job Linux e Windows: non è possibile dedurre da quell'esecuzione alcun esito di build o test.
- Gli esiti della Run #522 sono soltanto un riferimento storico della baseline.
- La Run #544 dimostra il superamento dei controlli configurati su Linux e Windows per lo SHA `1a3c5f4d24fd25d599596b1d16a7fc4006d0106b`; non attribuire a questa verifica un numero di test non letto dai log.
- I tre commit successivi di `main` devono essere riconciliati durante la preparazione della Pull Request, con particolare attenzione ai documenti ricreati dopo la loro eliminazione.

---

## 8. Prossima attività e autorizzazioni

**Passo immediato:** completare l'aggiornamento dei tre documenti di continuità sul branch `refactor/src-domain-layout`:
1. `CODEX_PROGRESS.md`;
2. `.github/workflows/benilla-classic-pipeline-revised-v3.md`;
3. `.github/workflows/refactoring.md`.

**Passo successivo:** confrontare nuovamente il branch con `main`, verificare i tre commit documentali successivi e preparare l'integrazione preservando tutte le informazioni aggiornate.

Durante la preparazione:
- non modificare direttamente `main`;
- non sovrascrivere i documenti più aggiornati con versioni precedenti;
- non eliminare documenti o contenuti senza autorizzazione;
- non creare una Pull Request finché il confronto e la riconciliazione non sono stati verificati;
- non eseguire il merge senza autorizzazione esplicita;
- non avviare la Fase 4 senza autorizzazione.

Dopo l'integrazione:
1. verificare il nuovo HEAD di `main`;
2. controllare che la struttura di `src/` sia quella verificata dalla Run #544;
3. confermare che roadmap e registro di avanzamento siano presenti e coerenti;
4. aggiornare questo checkpoint con il commit di merge e lo stato effettivo del repository;
5. preparare il branch dedicato alla prossima attività soltanto dopo aver concordato i requisiti.

La Fase 3.5 è completata. La Fase 4 resta in attesa dell'integrazione del refactoring in `main` e dell'autorizzazione dell'utente.
