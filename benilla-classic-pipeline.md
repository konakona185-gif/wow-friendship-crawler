# Pipeline dei contenuti e struttura del progetto

## 1. Identità del progetto

Questo repository contiene un Action RPG cooperativo in terza persona, incentrato sull'esplorazione di dungeon e sugli scontri di gruppo.

Il progetto prende ispirazione da World of Warcraft Classic e da Fellowship, ma non intende riprodurre un client o un server di World of Warcraft.

### Riferimenti tecnici

- Repository: https://github.com/konakona185-gif/wow-friendship-crawler
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

Moduli attualmente presenti:

- `src/combat/damage.rs`
- `src/combat/death.rs`
- `src/combat/effect_magnitude.rs`
- `src/combat/effect_resolution.rs`
- `src/combat/enemy_ai.rs`
- `src/combat/enemy_attacks.rs`
- `src/combat/events.rs`
- `src/combat/health.rs`
- `src/combat/healing.rs`
- `src/combat/player_attacks.rs`
- `src/combat/resources.rs`
- `src/combat/stats.rs`
- `src/combat/target_indicator.rs`
- `src/combat/target_selection.rs`

Sottomoduli:

- `src/combat/damage/critical.rs`
- `src/combat/damage/mitigation.rs`

Responsabilità principali:

- Statistiche, attributi e livelli dei combattenti.
- Salute e risorse.
- Richieste e risoluzioni di danno.
- Calcolo dei colpi critici.
- Mitigazione tramite armatura e resistenze.
- Calcolo della magnitudine degli effetti.
- Preparazione delle operazioni da eseguire.
- Contesto degli eventi di combattimento.
- Comportamento dei nemici e selezione dei bersagli.

Il combattimento attuale contiene ancora alcune meccaniche provvisorie, da conservare fino alla loro sostituzione con i sistemi definitivi.

I moduli con responsabilità differenti devono rimanere separati. I file che superano circa 1.000 righe dovranno essere valutati per una suddivisione modulare, evitando frammentazioni artificiali.

### Contenuti

Percorso principale: `src/content.rs`

Moduli:

- `src/content/ability.rs`
- `src/content/combat_attribute.rs`
- `src/content/database.rs`
- `src/content/damage_school.rs`
- `src/content/effect.rs`
- `src/content/id.rs`
- `src/content/loader.rs`
- `src/content/metadata.rs`
- `src/content/pack.rs`
- `src/content/registry.rs`
- `src/content/resource.rs`
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
- Definizioni delle scuole di danno.
- Definizioni degli attributi di combattimento.
- Definizioni delle risorse.
- Definizioni degli effetti.
- Definizioni delle abilità.
- Definizioni degli alberi e dei talenti.
- Database centrale dei contenuti.
- Registrazione e ricerca dei contenuti.
- Validazione dei riferimenti tra contenuti.
- Caricamento RON con controllo degli errori.
- Caricamento transazionale: i dati vengono applicati al database soltanto dopo aver superato le verifiche.
- Supporto di `ContentDatabase` come `Resource` Bevy, per renderlo accessibile ai sistemi ECS.

Non sono ancora presenti:

- Pacchetti RON effettivi completi con i contenuti di gioco.
- Caricamento automatico dei pacchetti nella fase di avvio del gioco.
- Integrazione del database caricato con i sistemi di combattimento.
- Ricaricamento dinamico dei contenuti.
- Regole definitive per sovrascrittura e precedenza dei pacchetti.
- Dati originali completi di abilità e talenti Classic.

La distinzione tra database registrabile e database effettivamente caricato nel runtime deve essere mantenuta: il fatto che `ContentDatabase` sia una `Resource` non significa che i pacchetti siano già caricati automaticamente.

## 6. Stato attuale del sistema di combattimento

Le fondamenta del sistema di combattimento sono state consolidate.

### Statistiche e attributi

Sono presenti:

- `CombatStats`
- `CombatAttributes`
- `CombatLevel`

Gli attributi sono identificati tramite `ContentId` e memorizzati in una struttura estendibile.

Gli attributi non registrati restituiscono attualmente valore zero.

### Salute e risorse

Sono presenti:

- `HealthPool`
- `CombatResources`
- `ResourcePool`
- `HealingRequest`
- `HealingResolution`
- `ResourceRequest`
- `ResourceResolution`

Le pool supportano validazione, aggiornamento dei valori e applicazione di operazioni effettive.

`HealthPool` costituisce il riferimento autorevole per la salute. `CombatStats` mantiene ancora i valori legacy e deve essere considerato un sistema provvisorio di compatibilità, non una seconda fonte indipendente di verità.

### Scuole di danno

Le scuole fondamentali previste sono:

- Physical
- Holy
- Shadow
- Nature
- Frost
- Arcane
- Fire

È previsto anche il supporto alle scuole ibride e composite.

Le 21 combinazioni ibride a due componenti sono:

- Holystrike
- Shadowstrike
- Stormstrike
- Icestrike
- Spellstrike
- Firestrike
- Twilight
- Anima
- Holyfrost
- Divine
- Radiant
- Plague
- Entropy
- Curse
- Incantation
- Rainstorm
- Astral
- Firestorm
- Spellfrost
- Freezerburn
- Spellfire

Sono inoltre previsti gruppi compositi:

- Chaos
- Cosmic
- Elemental
- Blood
- Chi
- Magical

Le combinazioni e le relative denominazioni seguono la matrice progettuale stabilita.

### Regole di mitigazione

Per le scuole ibride e composite si applica esclusivamente la mitigazione più elevata tra i componenti. Le resistenze non devono essere sommate.

Per le combinazioni Physical + magia, la mitigazione fisica tramite armatura deve essere confrontata con quella magica in termini di percentuale effettiva, non confrontando direttamente i valori grezzi.

Formula dell'armatura:

`armor / (armor + 85 * attacker_level + 400)`

La mitigazione massima è del 75%.

Formula della resistenza:

`min(0.75, max(resistance, 0) / (max(attacker_level, 20) * 5) * 0.75)`

La resistenza è deterministica: non vengono introdotti tiri casuali per le mitigazioni parziali.

A livello 60:

- 100 resistenza corrisponde al 25%.
- 200 resistenza corrisponde al 50%.
- 300 resistenza corrisponde al 75%.

La resistenza Holy deve essere supportata tramite un attributo dedicato `holy_resistance`.

### Colpi critici

Il modulo `critical.rs` contiene il calcolo dei colpi critici.

La funzione riceve la probabilità, il moltiplicatore e il tiro già generato, determinando il risultato. La generazione casuale del tiro non è responsabilità di questo modulo.

La probabilità deve rimanere nell'intervallo 0–100 e il moltiplicatore deve essere almeno 1.

### Stato dell'integrazione

I modelli di danno, mitigazione, guarigione e risorse esistono e dispongono di test dedicati.

Le meccaniche provvisorie di attacco non sono ancora state sostituite con la pipeline definitiva.

Non applicare automaticamente critici, mitigazioni o altre regole agli attacchi segnaposto prima che il sistema definitivo delle abilità e degli effetti sia pronto.

## 7. Meccaniche provvisorie del combattimento

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

Devono rimanere operative fino a quando i rispettivi sistemi definitivi non saranno disponibili e testati.

## 8. Stato della compilazione automatica

Ultima verifica completa registrata come superata:

- Flusso di lavoro: numero 172.
- Identificativo: `41326117971`.
- Revisione: `18cb9c2050ebd111a85e27f4acc3196c36c01140`.
- Esito: superato.

Sono stati superati sia i controlli Linux sia quelli Windows, comprendenti compilazione in modalità release, test unitari, test senza interfaccia grafica e caricamento degli artefatti.

Riferimento: https://github.com/konakona185-gif/wow-friendship-crawler/actions/runs/41326117971

Questo risultato certifica esclusivamente quella specifica revisione. Le modifiche successive devono essere verificate tramite i rispettivi flussi di lavoro.

Non considerare una CI superata finché il relativo workflow sulla revisione esatta non riporta `success`.

## 9. Percorso di sviluppo

### Fase 1 — Struttura dei contenuti

**Stato: completata**

Realizzazione del sistema di identificazione, definizione, registrazione, validazione e caricamento dei pacchetti.

Sono stati implementati:

- Identificatori e metadati.
- Manifesti e dipendenze.
- Database centrale.
- Registrazione e ricerca.
- Validazione dei riferimenti.
- Loader RON.
- Caricamento transazionale.
- Definizioni di abilità, effetti, risorse, scuole di danno, attributi e talenti.

### Fase 2 — Fondamenta del combattimento

**Stato: completata**

Realizzazione del modello centrale e modulare per le regole di combattimento.

Sono stati implementati:

- Statistiche dei personaggi e dei nemici.
- Attributi di combattimento.
- Livelli dei combattenti.
- Salute e risorse.
- Scuole di danno.
- Scuole composite.
- Armatura e resistenze.
- Calcolo dei colpi critici.
- Richieste e risoluzioni del danno.
- Guarigione.
- Applicazione delle risorse.
- Contesto degli eventi di combattimento.
- Test di validazione e comportamento dei modelli.

La fase è conclusa a livello di fondamenta tecniche. L'integrazione operativa con le abilità e con gli attacchi effettivi sarà sviluppata nelle fasi successive, senza modificare prematuramente le meccaniche provvisorie.

### Fase 3 — Sistema di esecuzione degli effetti

**Stato: in corso**

Implementazione di un sistema centralizzato e modulare capace di trasformare le definizioni degli effetti in operazioni valide e di applicarle realmente alle entità del gioco.

Componenti già realizzati:

- Calcolo della magnitudine degli effetti tramite attributi e coefficienti di scaling.
- Risoluzione delle definizioni degli effetti.
- Preparazione di richieste di danno, guarigione e risorse.
- Risoluzione dei bersagli caster, selezionati e ad area.
- Validazione delle selezioni e prevenzione dei bersagli duplicati.
- Rappresentazione delle operazioni risolte.
- Integrazione di `ContentDatabase` come Resource Bevy.

Attività ancora da completare:

1. Caricamento effettivo dei pacchetti di contenuti nel runtime.
2. Collegamento del database ai sistemi di esecuzione.
3. Applicazione effettiva del danno alle `HealthPool`.
4. Integrazione della mitigazione con scuole di danno, armatura, resistenze e livello dell'attaccante.
5. Applicazione effettiva delle guarigioni.
6. Applicazione effettiva delle modifiche alle risorse.
7. Sistema di applicazione e rimozione di buff e debuff.
8. Gestione delle durate e degli stack delle aure.
9. Gestione degli effetti periodici.
10. Gestione della minaccia.
11. Gestione delle interruzioni.
12. Gestione degli spostamenti.
13. Gestione delle evocazioni.
14. Gestione degli effetti personalizzati.
15. Integrazione con il contesto e i trigger degli eventi di combattimento.
16. Test end-to-end delle operazioni sulle entità ECS.

Per gli effetti ad area il resolver riceve già gli enti selezionati, ma resta da realizzare l'integrazione con il sistema che individua effettivamente i bersagli in base a posizione, squadra e raggio.

La fase sarà considerata completa soltanto quando le operazioni risolte potranno essere applicate concretamente e correttamente nel mondo di gioco.

### Fase 4 — Sistema di esecuzione delle abilità

**Stato: da iniziare**

Realizzazione del sistema che gestisce l'utilizzo delle abilità e la loro integrazione con il sistema degli effetti.

Dovrà gestire:

- Attivazione delle abilità.
- Selezione e validazione dei bersagli.
- Distanza e portata.
- Costi delle risorse.
- Tempi di recupero.
- Tempi di lancio.
- Canalizzazioni.
- Interruzioni.
- Abilità passive.
- Attivazioni basate su eventi.
- Prerequisiti e restrizioni di classe.
- Collegamento agli effetti definiti nei pacchetti di contenuti.

Le abilità dovranno essere completamente data-driven quando supportate dal motore.

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
- Scuole di danno.
- Tempi di recupero e lancio.
- Comportamenti specifici delle abilità.

I contenuti dovranno essere organizzati in pacchetti RON, senza codificare singolarmente ogni abilità nel motore.

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
- Comportamenti specifici dei boss.
- Interazioni con gli effetti e le abilità dei giocatori.

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
- Influenza dell'equipaggiamento sugli attributi e sulle abilità.

### Fase 9 — Sistema cooperativo

**Stato: parzialmente avviata**

Consolidamento dei sistemi necessari al gioco di gruppo.

Sono già presenti le prime strutture per:

- Fellowship.
- Membri del gruppo.
- Ruoli.
- Identificazione dei giocatori.
- Gestione delle appartenenze.

Restano da sviluppare:

- Gestione completa dei giocatori.
- Interazioni tra membri.
- Sinergie tra classi.
- Condivisione delle attività.
- Gestione dei gruppi e degli incontri.
- Sincronizzazione dello stato di gioco.
- Infrastruttura multiplayer.
- Gestione della connessione e della disconnessione.
- Autorità e coerenza dello stato di combattimento.

La presenza della Fellowship non equivale a un multiplayer di rete già implementato.

### Fase 10 — Espansione dei contenuti

**Stato: pianificata**

Estensione del sistema per consentire l'aggiunta di contenuti originali e personalizzati, nonché di elementi ispirati o retroportati da espansioni successive.

L'architettura dovrà consentire di:

- Aggiungere nuove abilità.
- Aggiungere nuovi talenti e specializzazioni.
- Introdurre nuove scuole di danno.
- Introdurre nuove risorse.
- Creare nuovi effetti.
- Estendere le meccaniche già supportate.
- Mantenere più insiemi di contenuti.
- Definire esplicitamente dipendenze e precedenze.
- Gestire sovrascritture e varianti dei contenuti.

WoW Classic 1.12.1 deve rimanere il punto di partenza, non il limite del progetto.

### Fase 11 — Rifinitura e ottimizzazione

**Stato: pianificata**

Ottimizzazione delle prestazioni, rifinitura dell'interfaccia, feedback visivi e sonori, bilanciamento e miglioramenti generali.

Comprenderà:

- Ottimizzazione ECS.
- Riduzione delle allocazioni e delle operazioni superflue.
- Miglioramento della gestione delle risorse.
- Rifinitura dell'interfaccia.
- Feedback visivi e sonori.
- Bilanciamento delle classi.
- Bilanciamento dei dungeon.
- Test di regressione.
- Miglioramenti dell'esperienza cooperativa.

## 10. Regole operative dello sviluppo

- Modificare un solo file alla volta.
- Leggere sempre il file completo prima di intervenire.
- Accorpare le correzioni correlate quando riguardano lo stesso file.
- Per modifiche multilinea fornire sempre il file completo aggiornato.
- Per una modifica di una sola riga fornire soltanto la riga e indicare dove sostituirla.
- Non aggiungere commenti al codice né righe di commento che iniziano con `#`.
- Suddividere i moduli che superano circa 1.000 righe quando esistono responsabilità distinte che giustificano la separazione.
- Evitare frammentazioni artificiali e mantenere API e comportamento.
- Preservare le meccaniche già funzionanti.
- Non modificare una meccanica esistente senza spiegare prima le conseguenze e ottenere l'approvazione.
- Preferire sistemi modulari e riutilizzabili.
- Evitare duplicazioni di logica.
- Mantenere separati contenuti, regole di gioco e implementazione tecnica.
- Non applicare nuove formule agli attacchi provvisori prima di avere completato il relativo sistema definitivo.
- Accorpare le modifiche compatibili prima di avviare una nuova verifica automatica.
- Verificare i commit e i workflow GitHub sulla revisione esatta.
- Non considerare una CI superata finché il relativo workflow non riporta `success`.
- Dopo un gruppo significativo di modifiche aggiornare questo documento e fornire un nuovo prompt di continuità.

## 11. Prossimi passaggi operativi

L'ordine di lavoro attuale è:

1. Completare il caricamento effettivo dei pacchetti RON nel runtime.
2. Rendere il database dei contenuti accessibile ai sistemi di combattimento.
3. Implementare l'applicazione effettiva delle operazioni risolte alle entità ECS.
4. Collegare correttamente danni, mitigazioni, guarigioni e risorse.
5. Implementare buff, debuff, aure e relativi timer.
6. Implementare effetti periodici.
7. Integrare minaccia, interruzioni, spostamenti ed evocazioni.
8. Collegare la gestione degli effetti agli eventi di combattimento.
9. Completare i test di integrazione della fase 3.
10. Avviare la fase 4 dedicata all'esecuzione delle abilità.
11. Sostituire progressivamente le meccaniche provvisorie soltanto quando le alternative definitive saranno pronte.

Le attività dovranno procedere un file alla volta, evitando di anticipare integrazioni che richiederebbero meccaniche ancora mancanti.

## 12. Continuità del progetto

Il repository è la fonte di verità per lo stato effettivo del codice.

Questo documento serve a conservare la direzione architetturale, le decisioni di progettazione, le fasi di sviluppo e il relativo stato.

Prima di ogni intervento occorre controllare la situazione reale del repository, senza assumere che le informazioni riportate in questo documento siano necessariamente aggiornate dopo gli ultimi commit.

La roadmap deve essere aggiornata quando viene completata una fase significativa, viene introdotta una componente architetturale importante, cambia una decisione progettuale o viene modificata sostanzialmente la direzione dello sviluppo.

L'obiettivo finale è costruire un sistema di gioco modulare, coerente e ampliabile, nel quale i contenuti originali costituiscano una base solida sulla quale poter sviluppare liberamente nuove possibilità.
