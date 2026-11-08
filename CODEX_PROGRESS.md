# Registro di avanzamento Codex

## 1. Stato operativo corrente

**Ultimo aggiornamento: 2026-11-08**

- Progetto: `WoW-Friendship-Crawler` (`konakona185-gif/wow-friendship-crawler`).
- Branch di lavoro: `refactor/src-domain-layout`.
- Ultimo HEAD remoto verificato prima di questo aggiornamento: `f15b9b02d84a165b2a67eb074c1d1c168b17c821`.
- Ultimo commit: `Update refactoring.md`.
- `main` non è stato modificato.
- Fase strutturale 3.5: Gruppi 1–6 completati e sottoposti ad audit statico.
- Gruppo 7: verifica tecnica finale in corso. La prima GitHub Actions Run #536 è fallita durante il controllo di formattazione; la correzione è stata pubblicata e la nuova CI è ancora da eseguire.
- Fase 4: bloccata fino al superamento della verifica tecnica finale sullo SHA corretto.

### Ultimo stato della verifica CI

La Run #536 è stata avviata manualmente sul branch `refactor/src-domain-layout`, allo SHA `0cfcf2a61fff40500c7f9aac7ebec7c50cae7c77`.

- Esito: fallimento durante `cargo fmt --all -- --check`.
- Causa: direttiva `#[cfg(test)]` rimasta orfana in `src/combat/effects/dispatcher/tests/reactive_runtime_tests.rs`, alla riga 488.
- I job di compilazione e test Linux e Windows sono stati saltati perché dipendenti dal controllo preliminare.
- Nessuna compilazione, suite di test release o verifica headless è stata completata in quella run.
- Nessun artifact Linux o Windows è stato prodotto.

La correzione è stata pubblicata con il commit:

`b69aefcf7efe844d8c9bf32227bf04ec6cfca2ba`

Messaggio: `Remove unnecessary lines from reactive_runtime_tests.rs`.

È stata rimossa esclusivamente la direttiva `#[cfg(test)]` orfana. Gli 11 test del file e le suite di test sibling sono stati preservati.

Dopo la correzione sono stati aggiornati:
- `.github/workflows/benilla-classic-pipeline-revised-v3.md`, commit `62fac5fb90a2dcb414106753374ade67e5c7c9d2`;
- `.github/workflows/refactoring.md`, commit `f15b9b02d84a165b2a67eb074c1d1c168b17c821`.

La nuova GitHub Actions sullo SHA corretto è ancora pendente. Il fallimento della Run #536 non dimostra un errore di compilazione o nei test: quei job non sono stati raggiunti.

### Prossimo passaggio

1. Salvare e verificare questo aggiornamento di `CODEX_PROGRESS.md`.
2. Controllare che roadmap, registro di refactoring e questo documento descrivano lo stesso stato.
3. Ottenere l'autorizzazione esplicita prima di avviare una nuova GitHub Actions `build.yml` tramite `workflow_dispatch` sul branch `refactor/src-domain-layout`.
4. In caso di esito verde, registrare SHA, run, piattaforme, test, avvio headless e artifact effettivamente verificati.
5. Considerare conclusa la Fase 3.5 soltanto dopo aver controllato l'intero risultato.

Non avviare la nuova run automaticamente durante il semplice aggiornamento documentale.

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

Limite dell'audit: senza compilazione non è possibile confermare definitivamente la risoluzione dei nomi Rust, delle visibilità o il comportamento runtime.

---

## 6. Gruppo 7 – Verifica tecnica finale

<ins>Stato: IN CORSO – PRIMA CI FALLITA, CORREZIONE PUBBLICATA, NUOVA CI PENDENTE.</ins>

### Workflow

File: `.github/workflows/build.yml`.

Il controllo di formattazione è stato modificato in precedenza: `cargo fmt --all -- --check` è ora eseguito come passo diretto e bloccante, non come warning.

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

### Correzione pubblicata

Commit:

`b69aefcf7efe844d8c9bf32227bf04ec6cfca2ba`

Messaggio:
`Remove unnecessary lines from reactive_runtime_tests.rs`

Modifica:
- rimossa esclusivamente la direttiva `#[cfg(test)]` orfana;
- preservati tutti gli 11 test del file;
- preservati i test sibling;
- nessuna modifica funzionale.

### Aggiornamenti documentali successivi

Roadmap principale:
- file: `.github/workflows/benilla-classic-pipeline-revised-v3.md`;
- commit: `62fac5fb90a2dcb414106753374ade67e5c7c9d2`.

Piano operativo di refactoring:
- file: `.github/workflows/refactoring.md`;
- commit: `f15b9b02d84a165b2a67eb074c1d1c168b17c821`;
- modifica registrata: aggiornamento dello stato della Fase 5, della Run #536 e della correzione pubblicata.

Ultimo HEAD remoto verificato prima dell'aggiornamento di questo registro:

`f15b9b02d84a165b2a67eb074c1d1c168b17c821`

La nuova GitHub Actions sullo SHA successivo alla correzione non è ancora stata eseguita.

### Criteri per concludere il Gruppo 7

Il gruppo è completo soltanto quando:
- `cargo fmt --all -- --check` supera il controllo;
- `cargo check --all-targets` termina correttamente;
- build release Linux e Windows completate;
- test release Linux e Windows completati;
- avvio headless Linux e Windows verificato;
- artifact attesi caricati e verificati;
- inventario dei test confrontato con il riferimento storico di 660 test, spiegando eventuali differenze motivate;
- diff finale ispezionato per escludere modifiche funzionali non autorizzate;
- SHA esatto del risultato finale registrato nella roadmap e in questo documento.

Solo dopo questi controlli è possibile dichiarare completata la Fase 3.5 e sbloccare la Fase 4.

---

## 7. Limitazioni tecniche note

- Cargo, rustfmt e gli strumenti Rust non sono disponibili nell'ambiente locale Windows utilizzato durante la preparazione.
- Non sono stati installati strumenti Rust né modificate configurazioni dell'ambiente.
- Le verifiche statiche non sostituiscono formattazione, compilazione o test.
- La Run #536 è fallita prima dei job Linux e Windows: non è possibile dedurre da quell'esecuzione alcun esito di build o test.
- Gli esiti della Run #522 sono soltanto un riferimento storico della baseline.
- Non dichiarare mai verde la nuova struttura sulla base della sola correzione del file: la verifica completa resta obbligatoria.

---

## 8. Prossima attività e autorizzazioni

**Passo immediato:** completare e verificare questo aggiornamento documentale.

**Passo successivo:** chiedere l'autorizzazione dell'utente per avviare manualmente `.github/workflows/build.yml` sul branch `refactor/src-domain-layout`, usando lo SHA corrente dopo l'aggiornamento dei documenti.

Durante l'esecuzione:
- non cambiare branch;
- non modificare `main`;
- non avviare una Pull Request;
- non effettuare merge;
- non modificare codice Rust senza aver prima esaminato l'eventuale errore;
- in caso di fallimento, individuare e raggruppare le correzioni necessarie prima di una nuova esecuzione, evitando verifiche ridondanti.

Dopo una CI interamente verde:
1. registrare risultati, numero dei test, SHA, link alla run e artifact;
2. aggiornare la roadmap principale per chiudere la Fase 3.5;
3. valutare l'eliminazione del documento temporaneo `.github/workflows/refactoring.md`, dopo aver conservato le informazioni ancora utili;
4. non avviare la Fase 4 senza autorizzazione.

La Fase 4 resta bloccata fino al completamento e alla verifica dell'intero Gruppo 7.

