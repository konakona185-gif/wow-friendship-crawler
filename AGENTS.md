# Istruzioni permanenti per il repository

## Branch e sicurezza Git

- Lavora esclusivamente sul branch `refactor/src-domain-layout`. Prima di ogni gruppo di modifiche verifica il branch attivo e lo stato del working tree; se il branch è diverso o lo stato non è chiaro, fermati senza cambiare branch.
- Non passare mai a `main` e non modificarlo.
- Non eseguire push, merge o creare/aggiornare Pull Request senza autorizzazione esplicita dell'utente.
- Non eseguire operazioni Git distruttive, incluse `reset --hard`, `clean` o checkout forzati.
- Non eliminare o sovrascrivere file senza averne prima verificato il contenuto.
- Un commit locale di sicurezza può essere preparato solo dopo aver completato e verificato un gruppo coerente e dopo aver chiesto l'autorizzazione dell'utente.

## Ambito del refactoring

- Prima di iniziare una sessione, leggi la roadmap `.github/workflows/benilla-classic-pipeline-revised-v3.md` e il piano operativo `.github/workflows/refactoring.md`.
- Il refactoring di questa fase è esclusivamente strutturale: conserva il comportamento del gioco, i dati, le API necessarie e tutti i test esistenti. Non introdurre modifiche funzionali senza una decisione esplicita dell'utente.
- Segui la mappatura dei file e le checklist in `.github/workflows/refactoring.md`. Prima di spostare moduli, controlla dichiarazioni `mod`, import, re-export, visibilità, `use super::*`, attributi `#[path]`, macro `include_str!`/`include_bytes!` e riferimenti ai percorsi.
- Mantieni i file Rust, inclusi i test, entro 1.000 righe quando la suddivisione in unità coerenti è possibile. Oltre 1.000 righe valuta e documenta la suddivisione; oltre 1.500 righe serve una motivazione tecnica e l'approvazione esplicita dell'utente.
- Procedi per gruppi piccoli, coerenti e verificabili. Non dichiarare completato un gruppo solo perché il codice o i documenti sono stati scritti.

## Continuità e verifiche

- Mantieni aggiornato `CODEX_PROGRESS.md` nella root. Prima di ciascun gruppo registra branch, stato Git e SHA; dopo il gruppo registra attività completate, file creati/modificati/rimossi, attività incomplete, test eseguiti e risultati, errori noti, prossimo passo preciso e ultimo checkpoint Git.
- Nel registro distingui sempre tra modifiche completate e verificate, modifiche scritte ma non verificate, modifiche incomplete, test superati, test falliti e test non eseguiti.
- In caso di interruzione, riprendi leggendo prima queste istruzioni, la roadmap, il piano operativo e `CODEX_PROGRESS.md`; poi verifica branch, stato Git e SHA. Riparti dall'ultimo checkpoint documentato e non presumere che le attività non verificate siano concluse.
- Non iniziare la Fase 4 del progetto finché il refactoring strutturale non è completato e verificato secondo le checklist.
