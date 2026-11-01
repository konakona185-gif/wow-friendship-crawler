# Pipeline dei contenuti e struttura del progetto

## 1. Identità del progetto

Questo repository contiene un Action RPG cooperativo in terza persona, incentrato sull'esplorazione di dungeon e sugli scontri di gruppo.

Il progetto prende ispirazione da World of Warcraft Classic e da Fellowship, ma non intende riprodurre un client o un server di World of Warcraft.

### Riferimenti tecnici

- Linguaggio: Rust.
- Motore: Bevy 0.18.1.
- Fisica: Avian3D 0.6.
- Sistema di navigazione: Bevy Landmass.
- Base di riferimento dei contenuti: World of Warcraft Classic 1.12.1, build 5875.
- Client di riferimento: Benilla.
- Revisione Benilla utilizzata: `135ce8889217bf268d1dadbbabcba56df80f3108`.
- Unità di misura: 1 unità di gioco corrisponde a 1 yard di World of Warcraft.

## 2. Fondamenti del gioco

Il personaggio parte direttamente dal livello 60. Il progetto è focalizzato esclusivamente sui dungeon e sulle attività cooperative: non è prevista una progressione tradizionale dal livello 1 al 60 né una campagna open world basata su missioni.

### Razze previste

- Human
- Dwarf
- Night Elf
- Gnome
- Orc
- Undead
- Tauren
- Troll

### Classi previste

- Warrior
- Paladin
- Hunter
- Rogue
- Priest
- Shaman
- Mage
- Warlock
- Druid

Sono previste alcune combinazioni personalizzate non presenti nella versione originale di riferimento, tra cui Dwarf Shaman e Tauren Paladin.

### Ruoli del gruppo

I ruoli del gruppo sono indipendenti dalla classe:

- Tank
- Healer
- DPS

Il Druid deve poter ricoprire tutti e tre i ruoli. Il sistema dovrà consentire anche ad altre classi di essere configurate secondo le regole e le meccaniche previste dal progetto.

## 3. Principi architetturali

L'architettura deve essere modulare ed espandibile.

I contenuti di gioco devono essere separati dal codice che ne gestisce il funzionamento. L'obiettivo è poter aggiungere, modificare o sostituire abilità, talenti, effetti, equipaggiamenti, nemici e altri contenuti senza dover riscrivere continuamente il motore.

### Formato dei contenuti

Il sistema utilizza:

- RON per la definizione dei contenuti.
- Serde per la serializzazione e deserializzazione.
- Rust per la gestione delle regole e delle meccaniche.

La base Classic 1.12.1 costituisce il primo insieme di contenuti, non un limite definitivo.

In futuro dovrà essere possibile introdurre:

- Nuove abilità e passivi.
- Nuovi talenti e alberi dei talenti.
- Nuove classi e specializzazioni.
- Nuovi effetti e meccaniche.
- Contenuti personalizzati.
- Contenuti ispirati o retroportati da espansioni successive.
- Modifiche e varianti delle meccaniche originali.

Le regole di sostituzione, estensione e precedenza tra contenuti dovranno essere definite prima di implementare un sistema di sovrascrittura completo.

Una distinzione fondamentale: i contenuti supportati dal motore devono essere configurabili attraverso i dati; una meccanica completamente nuova richiederà invece un'implementazione specifica nel motore, da rendere successivamente riutilizzabile.

## 4. Organizzazione attuale del codice

### Personaggi

Percorso principale: `src/character.rs`

Moduli:

- `src/character/build.rs`
- `src/character/class.rs`
- `src/character/race.rs`
- `src/character/tests.rs`

La configurazione del personaggio comprende razza e classe. Il ruolo Fellowship è gestito separatamente.

### Combattimento

Percorso principale: `src/combat.rs`

Moduli:

- `src/combat/death.rs`
- `src/combat/enemy_ai.rs`
- `src/combat/enemy_attacks.rs`
- `src/combat/player_attacks.rs`
- `src/combat/target_indicator.rs`
- `src/combat/target_selection.rs`

Il combattimento attuale contiene ancora alcune meccaniche provvisorie, da conservare fino alla loro sostituzione con i sistemi definitivi.

### Contenuti

Percorso principale: `src/content.rs`

Moduli:

- `src/content/ability.rs`
- `src/content/database.rs`
- `src/content/effect.rs`
- `src/content/id.rs`
- `src/content/loader.rs`
- `src/content/metadata.rs`
- `src/content/pack.rs`
- `src/content/registry.rs`
- `src/content/talent.rs`

Sottomoduli del database:

- `src/content/database/lookup.rs`
- `src/content/database/registration.rs`
- `src/content/database/validation.rs`
- `src/content/database/tests.rs`

## 5. Stato attuale del sistema dei contenuti

La struttura di base è stata realizzata e integrata.

Sono già presenti:

- Identificatori univoci dei contenuti con namespace.
- Metadati dei contenuti.
- Manifesti dei pacchetti.
- Registro dei pacchetti e ordinamento delle dipendenze.
- Definizioni degli effetti.
- Definizioni delle abilità.
- Definizioni degli alberi e dei talenti.
- Database centrale dei contenuti.
- Registrazione, ricerca e validazione.
- Caricamento RON con controllo degli errori.
- Caricamento transazionale: i dati vengono applicati al database soltanto dopo aver superato le verifiche.

Non sono ancora presenti:

- Pacchetti RON effettivi con i contenuti di gioco.
- Caricamento automatico dei pacchetti da una cartella di gioco.
- Ricaricamento dinamico dei contenuti.
- Esecuzione effettiva degli effetti.
- Sistema completo di esecuzione delle abilità.
- Dati originali completi di abilità e talenti Classic.
- Regole definitive per sovrascrittura e precedenza dei pacchetti.

## 6. Meccaniche provvisorie del combattimento

Il sistema di combattimento contiene attualmente una prima implementazione temporanea di `ability_1`, differenziata in base al ruolo Fellowship.

### Tank

- Infligge 15 danni.
- Genera 20 risorse.
- Moltiplicatore della minaccia pari a 3.

### DPS

- Con almeno 30 risorse, ne consuma 30 e infligge 40 danni.
- Altrimenti infligge 10 danni e genera 15 risorse.
- Moltiplicatore della minaccia pari a 1.

### Healer

- Con almeno 20 risorse, ne consuma 20 e infligge 25 danni.
- Senza risorse sufficienti, l'abilità non viene eseguita.
- Moltiplicatore della minaccia pari a 1.

### Nemici

Valori provvisori:

- Aggro: 20.
- Distanza massima di inseguimento: 40.
- Portata corpo a corpo: 2,5.
- Danno: 5.
- Intervallo tra gli attacchi: 2 secondi.

Queste meccaniche sono segnaposto e non rappresentano il bilanciamento definitivo del gioco.

## 7. Stato della compilazione automatica

Ultima verifica completa registrata:

- Flusso di lavoro: numero 172.
- Identificativo: `41326117971`.
- Revisione: `18cb9c2050ebd111a85e27f4acc3196c36c01140`.
- Esito: superato.

Sono stati superati sia i controlli Linux sia quelli Windows, comprendenti compilazione in modalità release, test unitari, test senza interfaccia grafica e caricamento degli artefatti.

Riferimento: https://github.com/konakona185-gif/wow-friendship-crawler/actions/runs/41326117971

Questo risultato certifica quella specifica revisione. Le modifiche successive dovranno essere verificate tramite i rispettivi flussi di lavoro.

## 8. Percorso di sviluppo

### Fase 1 — Struttura dei contenuti

**Stato: completata**

Realizzazione del sistema di identificazione, definizione, registrazione, validazione e caricamento dei pacchetti.

### Fase 2 — Fondamenta del combattimento

**Stato: da iniziare**

Realizzazione di un modello centrale e modulare per:

- Statistiche dei personaggi e dei nemici.
- Salute e risorse.
- Scuole di danno.
- Calcolo degli attacchi e degli incantesimi.
- Armatura e resistenze.
- Colpi critici.
- Guarigione.
- Contesto degli eventi di combattimento.

### Fase 3 — Sistema di esecuzione degli effetti

**Stato: da iniziare**

Implementazione di un sistema centralizzato per gli effetti, comprendente:

- Danni.
- Guarigione.
- Modifica delle risorse.
- Potenziamenti e penalità.
- Effetti periodici.
- Minaccia.
- Interruzioni.
- Spostamenti.
- Evocazioni.
- Eventuali effetti personalizzati.

### Fase 4 — Sistema di esecuzione delle abilità

**Stato: da iniziare**

Il sistema dovrà gestire:

- Attivazione.
- Selezione e validazione dei bersagli.
- Distanza e portata.
- Costi delle risorse.
- Tempi di recupero.
- Tempi di lancio.
- Canalizzazioni.
- Interruzioni.
- Abilità passive.
- Attivazioni basate su eventi.

### Fase 5 — Contenuti originali Classic

**Stato: da iniziare**

Inserimento progressivo delle abilità e dei contenuti originali di Classic 1.12.1 per tutte le classi.

Dovranno essere rappresentati correttamente:

- Abilità attive.
- Abilità passive.
- Gradi delle abilità.
- Costi e requisiti.
- Restrizioni di classe.
- Prerequisiti.
- Effetti e interazioni.

### Fase 6 — Talenti e configurazioni

**Stato: da iniziare**

Implementazione dei 27 alberi dei talenti originali, tre per ciascuna delle nove classi.

Il sistema dovrà supportare:

- Gradi multipli per talento.
- Prerequisiti.
- Effetti per ogni grado.
- Sblocco di abilità.
- Distribuzione dei punti.
- Configurazioni complete da livello 60.
- Validazione dei requisiti di ciascuna configurazione.

Il sistema deve essere sufficientemente generico da supportare in futuro nuovi alberi e talenti personalizzati.

### Fase 7 — Nemici e dungeon

**Stato: da iniziare**

Realizzazione dei sistemi per:

- Tipologie di nemici.
- Statistiche e comportamenti.
- Abilità nemiche.
- Aggro e gestione della minaccia.
- Incontri con gruppi di nemici.
- Boss e relative fasi.
- Meccaniche degli scontri.
- Progressione e difficoltà dei dungeon.

### Fase 8 — Equipaggiamento e ricompense

**Stato: da iniziare**

Implementazione di:

- Tipologie di equipaggiamento.
- Statistiche degli oggetti.
- Requisiti e restrizioni.
- Affissi e proprietà.
- Tabelle di bottino.
- Ricompense dei dungeon.
- Gestione degli oggetti e dell'inventario.

### Fase 9 — Sistema cooperativo

**Stato: parzialmente avviata**

Consolidamento dei sistemi necessari al gioco di gruppo:

- Fellowship.
- Ruoli.
- Gestione dei giocatori.
- Interazioni tra membri.
- Sinergie tra classi.
- Condivisione delle attività.
- Infrastruttura multiplayer.

### Fase 10 — Espansione dei contenuti

**Stato: pianificata**

Estensione del sistema per consentire l'aggiunta di contenuti originali e personalizzati, nonché di elementi ispirati o retroportati da espansioni successive.

L'architettura dovrà consentire di mantenere più insiemi di contenuti e di definire in maniera esplicita le relative dipendenze e precedenze.

### Fase 11 — Rifinitura e ottimizzazione

**Stato: pianificata**

Ottimizzazione delle prestazioni, rifinitura dell'interfaccia, feedback visivi e sonori, bilanciamento e miglioramenti generali.

## 9. Regole operative dello sviluppo

- Modificare un solo file alla volta.
- Leggere sempre il file completo prima di intervenire.
- Accorpare le correzioni correlate quando riguardano lo stesso file.
- Suddividere i moduli troppo grandi quando questo migliora la manutenzione.
- Preservare le meccaniche già funzionanti.
- Non modificare una meccanica esistente senza spiegare prima le conseguenze e ottenere l'approvazione.
- Preferire sistemi modulari e riutilizzabili.
- Evitare duplicazioni di logica.
- Mantenere separati contenuti, regole di gioco e implementazione tecnica.
- Accorpare le modifiche compatibili prima di avviare una nuova verifica automatica.
- Dopo ogni gruppo di modifiche, controllare la compilazione e i test su Linux e Windows.

## 10. Prossimi passaggi

L'ordine di lavoro previsto è:

1. Conservare come riferimento la verifica completa superata nel flusso di lavoro 172.
2. Progettare il modello centrale delle statistiche e delle regole di combattimento.
3. Definire le strutture necessarie per danni, guarigione, risorse e scuole di danno.
4. Suddividere il codice in moduli piccoli e indipendenti.
5. Mantenere operative le meccaniche provvisorie finché non esistono le relative alternative definitive.
6. Realizzare il sistema di esecuzione degli effetti.
7. Realizzare il sistema di esecuzione delle abilità.
8. Iniziare l'inserimento dei contenuti Classic soltanto quando l'infrastruttura necessaria sarà pronta.

## 11. Continuità del progetto

Il repository è la fonte di verità per lo stato effettivo del codice.

Questo documento serve a conservare la direzione architetturale, le decisioni di progettazione, le fasi di sviluppo e il relativo stato.

Prima di ogni intervento occorre controllare la situazione reale del repository, senza assumere che le informazioni riportate in questo documento siano necessariamente aggiornate dopo gli ultimi commit.

L'obiettivo finale è costruire un sistema di gioco modulare, coerente e ampliabile, nel quale i contenuti originali costituiscano una base solida sulla quale poter sviluppare liberamente nuove possibilità.
