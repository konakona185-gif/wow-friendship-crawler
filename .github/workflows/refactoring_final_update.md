# Aggiornamento finale del refactoring – Fase 3.5

Ultimo aggiornamento: 9 novembre 2026  
Repository: `konakona185-gif/wow-friendship-crawler`  
Branch di lavoro: `refactor/src-domain-layout`

> **Istruzione:** questo documento è un addendum da accodare in fondo a `.github/workflows/refactoring.md`. Non sostituisce la mappatura storica dei percorsi, l'albero della struttura o le sezioni descrittive dei Gruppi 1–6. Le informazioni qui registrate prevalgono sugli stati provvisori precedenti relativi alla verifica finale.

---

## 1. Stato conclusivo

**Fase 3.5: COMPLETATA TECNICAMENTE.**

I Gruppi 1–6 sono stati completati e sottoposti ad audit statico. Il Gruppo 7 è stato completato con la GitHub Actions Run #544, interamente verde su Linux e Windows. La struttura riorganizzata è stata compilata, testata e avviata in modalità headless su entrambe le piattaforme.

L'integrazione in `main` non è ancora avvenuta. Deve essere preparata tramite Pull Request, preservando i tre commit documentali successivi presenti su `main`. La Fase 4 non deve iniziare prima dell'integrazione verificata e dell'autorizzazione esplicita dell'utente.

## 2. Verifica tecnica finale – Run #544

- Workflow: `.github/workflows/build.yml`
- Run ID: `42587661206`
- Numero run: `544`
- Evento: `workflow_dispatch`
- Branch: `refactor/src-domain-layout`
- SHA verificato: `1a3c5f4d24fd25d599596b1d16a7fc4006d0106b`
- Esito finale: `success`
- URL: https://github.com/konakona185-gif/wow-friendship-crawler/actions/runs/42587661206

### Controllo preliminare Rust

- `cargo fmt --all -- --check`: superato.
- Installazione delle dipendenze Linux: superata.
- `cargo check --all-targets`: superato.
- Riepilogo del controllo preliminare: superato.

### Linux

- Compilazione: superata.
- Test unitari: superati.
- Test headless: superati.
- Artifact `gioco-linux`: caricato correttamente.

### Windows

- Compilazione: superata.
- Test unitari: superati.
- Test headless: superati.
- Artifact `gioco-windows`: caricato correttamente.

### Artifact verificati

| Artifact | ID |
|---|---:|
| `gioco-linux` | `13661140684` |
| `gioco-windows` | `13660802759` |

La Run #544 costituisce la verifica tecnica conclusiva sullo SHA `1a3c5f4d24fd25d599596b1d16a7fc4006d0106b`. Non viene attribuito alla run un numero di test specifico che non sia stato letto nei log. Il conteggio statico dell'audit resta 660 occorrenze di test e 656 nomi distinti, invariati rispetto alla baseline.

## 3. Correzioni incluse nella verifica finale

### Correzione della formattazione dopo la Run #536

La Run #536 era fallita durante `cargo fmt --all -- --check` per una direttiva `#[cfg(test)]` orfana in:

`src/combat/effects/dispatcher/tests/reactive_runtime_tests.rs`

La direttiva è stata rimossa senza eliminare gli 11 test del file né le suite sibling. Correzione pubblicata con il commit:

`b69aefcf7efe844d8c9bf32227bf04ec6cfca2ba`

La Run #536 non aveva raggiunto la compilazione e i test Linux/Windows; il suo fallimento non costituisce un esito negativo della compilazione.

### Correzioni di visibilità e import

Durante la successiva compilazione sono emerse due diagnostiche relative a `summon_lifecycle_system`:

- E0364: re-export non consentito per una funzione con visibilità insufficiente.
- E0603: funzione non accessibile dal modulo chiamante.

In `src/combat/summon/lifecycle.rs` la visibilità della funzione è stata modificata da `pub(super)` a:

`pub(in crate::combat)`

Sono state inoltre corrette le visibilità dei sistemi e gli import necessari nei moduli e nei test interessati. La compilazione finale della Run #544 ha verificato le correzioni.

## 4. Audit strutturale complessivo

Risultati dell'audit del Gruppo 6:

- 88 file sorgente iniziali mappati in 129 destinazioni operative.
- 6 aggregatori combat aggiuntivi.
- Totale: 135 file Rust.
- Zero dichiarazioni `mod` senza destinazione.
- Zero destinazioni mancanti nei riferimenti a percorsi e macro controllati.
- 660 occorrenze di test e 656 nomi distinti, identici tra baseline e checkout.
- Multinsieme dei test invariato.
- 67 re-export Combat e 49 Content invariati.
- Ordine dei 14 sistemi di `CombatPlugin` invariato.
- Ordine di registrazione dei plugin in `main.rs` invariato.
- Nessun riferimento residuo ai vecchi percorsi dei moduli riorganizzati.
- File Rust più lungo: 893 righe.
- `git diff --check` superato.

I limiti dell'audit statico relativi alla risoluzione effettiva dei nomi e delle visibilità sono stati superati dalla compilazione completata nella Run #544.

## 5. Stato delle checklist

Le seguenti verifiche sono concluse:

- [x] Tutti gli 88 file iniziali hanno una destinazione registrata.
- [x] Nessun file sorgente o test è stato perso.
- [x] Moduli, import, re-export e visibilità sono stati verificati.
- [x] I test sono stati preservati secondo l'audit statico.
- [x] I file oltre 1.000 righe sono stati esaminati e la struttura è stata valutata secondo responsabilità e coesione.
- [x] Nessuna modifica intenzionale alle meccaniche di gioco è stata introdotta dal refactoring.
- [x] Formattazione Rust superata.
- [x] `cargo check --all-targets` superato.
- [x] Compilazione Linux superata.
- [x] Test unitari Linux superati.
- [x] Test headless Linux superati.
- [x] Compilazione Windows superata.
- [x] Test unitari Windows superati.
- [x] Test headless Windows superati.
- [x] Artifact Linux e Windows caricati e verificati.

**La Fase 3.5 è pertanto completata tecnicamente.** La revisione e l'integrazione tramite Pull Request sono attività successive, non verifiche tecniche ancora pendenti del Gruppo 7.

## 6. Preparazione dell'integrazione in `main`

Confronto GitHub verificato tra `main` e `refactor/src-domain-layout` prima degli aggiornamenti documentali finali:

- Branch di refactoring: 33 commit avanti rispetto a `main`.
- Branch di refactoring: 3 commit indietro rispetto a `main`.
- Stato del confronto: `diverged`.
- HEAD di `main` al controllo: `4c48fcca9358402426d2508d5e645b40e48f85db`.
- Merge base: `454874a358a18e7e565043d797e3c42ddc46323f`.

I tre commit successivi di `main` riguardano la gestione documentale:
1. eliminazione del vecchio `.github/workflows/benilla-classic-pipeline-revised-v2.md`;
2. eliminazione di `.github/workflows/refactoring.md`;
3. aggiunta della versione V3 della roadmap e della versione aggiornata di `refactoring.md`.

La versione del branch `refactor/src-domain-layout` contiene aggiornamenti successivi e informazioni operative da preservare. Prima della Pull Request occorre quindi:

1. completare e verificare gli aggiornamenti documentali sul branch di refactoring;
2. confrontare nuovamente `main` e `refactor/src-domain-layout` dopo tali aggiornamenti;
3. preservare le versioni correnti della roadmap e del registro, senza ripristinare documenti obsoleti;
4. esaminare il diff finale, distinguendo gli spostamenti strutturali dalle modifiche documentali;
5. preparare una Pull Request da `refactor/src-domain-layout` verso `main`;
6. revisionare la Pull Request e le verifiche automatiche associate;
7. eseguire il merge soltanto dopo approvazione esplicita dell'utente.

Non modificare direttamente `main`, non sincronizzare i branch automaticamente e non effettuare il merge senza approvazione.

## 7. Stato della Fase 4

**Fase 4 – Sistema di esecuzione delle abilità: NON INIZIATA.**

La Fase 4 rimane in attesa:
- dell'integrazione verificata della Fase 3.5 in `main`;
- dell'autorizzazione esplicita dell'utente;
- della definizione del branch dedicato e dei criteri di completamento della nuova fase.

## 8. Precedenza rispetto agli stati provvisori precedenti

Questo addendum aggiorna e sostituisce qualsiasi indicazione precedente in questo documento che riporti:

- Gruppo 7 ancora in corso;
- nuova CI sullo SHA corretto ancora pendente;
- compilazione o test Linux/Windows ancora da verificare;
- artifact non ancora prodotti;
- Fase 3.5 non ancora completata tecnicamente.

Le informazioni storiche sulle Run #536 e sui relativi errori restano conservate come cronologia; il risultato conclusivo è quello della Run #544.

La Fase 3.5 è completata tecnicamente. L'integrazione in `main` resta da effettuare tramite Pull Request. La Fase 4 non deve iniziare prima dell'integrazione e dell'autorizzazione dell'utente.
