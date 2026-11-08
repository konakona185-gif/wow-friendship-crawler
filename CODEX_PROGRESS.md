# Registro di avanzamento Codex

## Identificazione e checkpoint iniziale

- Progetto: `WoW-Friendship-Crawler` (`konakona185-gif/wow-friendship-crawler`).
- Branch di lavoro esclusivo: `refactor/src-domain-layout`.
- SHA iniziale della preparazione: `6989ae52c59b5e17264364c9926949e10aacf50a`.
- Stato Git iniziale: working tree pulito; branch allineato a `origin/refactor/src-domain-layout`.
- SHA iniziale di creazione del branch, riportato nei documenti progettuali: `454874a358a18e7e565043d797e3c42ddc46323f`.
- Fase attuale: preparazione e salvaguardia della Fase 3.5 (refactoring strutturale), prima degli spostamenti del codice. La Fase 4 del progetto resta bloccata fino al completamento e alla verifica della Fase 3.5.
- Ultimo checkpoint Git committato: `6989ae52c59b5e17264364c9926949e10aacf50a`.

## Stato verificato all'avvio

- Ricognizione di `src/`, dei test, delle dimensioni e delle dipendenze completata il 2026-11-08.
- Inventario osservato: 88 file Rust in 18 directory; dimensione misurata nella ricognizione: 948.467 byte.
- La roadmap riporta 913.840 byte per la baseline iniziale. Questa differenza è un elemento noto da riconciliare prima degli spostamenti; non è stata interpretata come modifica al codice.
- Quattro file superano 1.000 righe nella ricognizione: `combat/aura_runtime/tests.rs` (1.303), `combat/effect_execution.rs` (1.152), `combat/summon_integration_tests.rs` (1.137) e `combat/effect_runtime/tests.rs` (1.007).
- La ricognizione non ha modificato il codice né eseguito test.
- I documenti permanenti `AGENTS.md` e `CODEX_PROGRESS.md` sono stati creati in questa attività e verificati dopo la scrittura.

## Registro delle modifiche

### Completate e verificate

- Creazione di `AGENTS.md` con i vincoli permanenti di branch, sicurezza Git, ambito strutturale, lettura obbligatoria dei documenti, soglie dimensionali e continuità. Verificata rileggendo il file scritto.
- Creazione di `CODEX_PROGRESS.md` con il checkpoint iniziale, la ricognizione, i gruppi previsti, le regole di verifica e la procedura di ripresa. Verificata rileggendo il file scritto.

### Scritte ma non verificate

- Nessuna.

### Incomplete

- Riconciliare la differenza tra i 948.467 byte misurati e i 913.840 byte riportati come baseline prima degli spostamenti.
- Nessun gruppo di refactoring del codice è stato avviato.

## Gruppi di lavoro previsti

1. Preparazione: ricontrollare branch e working tree, riconciliare la baseline, verificare inventario e dimensioni prima degli spostamenti.
2. Conversione dei moduli top-level in directory con `mod.rs` e integrazione della enemy navigation.
3. Riordino del dominio combat per aree coerenti, preservando API, test e comportamento.
4. Suddivisione dei file grandi e dei relativi test per responsabilità, mantenendo fixture condivise.
5. Riordino di content, character, player, navigation, fellowship e interaction.
6. Audit completo di moduli, import, re-export, percorsi, test e dimensioni.
7. Verifica finale prevista dalla roadmap: formattazione, compilazione/test locali disponibili e CI Linux/Windows/headless sullo SHA finale.

Ogni gruppo richiede un controllo dello stato Git prima di iniziare e un aggiornamento di questo registro dopo il lavoro. I commit locali di sicurezza richiedono una richiesta di autorizzazione prima di essere creati. Push, merge e Pull Request richiedono autorizzazione esplicita.

## Test e verifiche

### Superati

- Nessun test è stato eseguito in questa attività.
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

- La dimensione totale osservata di `src/` differisce dalla baseline riportata nella roadmap; chiarire la metrica o la revisione di riferimento prima degli spostamenti.
- Spostare moduli può cambiare la risoluzione di `use super::*`, `#[path]`, `include_str!` e import relativi; audit richiesto per ogni gruppo.
- I re-export pubblici e `pub(crate)` del dominio combat devono restare disponibili ai chiamanti esistenti.

## Prossima attività autorizzata

Questa autorizzazione copriva esclusivamente la creazione e verifica di `AGENTS.md` e `CODEX_PROGRESS.md`; tali documenti sono completati e verificati. La prossima attività è attendere ulteriori istruzioni dell'utente. La preparazione operativa del primo gruppo di refactoring non è ancora autorizzata.
