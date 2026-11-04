
# Pipeline dei contenuti e struttura del progetto

Ultimo checkpoint: 4 novembre 2026  
Ultima revisione del codice verificata: `b0b727040bb31d2352dc928701450e945b55b79d`  
Fase attuale: Fase 3 — Sistema di esecuzione degli effetti  
Sotto-attività attuale: implementazione completa del sistema Summon.

## 1. Identità del progetto

Questo repository contiene un Action RPG cooperativo in terza persona, incentrato sull'esplorazione di dungeon e sugli scontri di gruppo.

Il progetto prende ispirazione da World of Warcraft Classic e da Fellowship, ma non intende riprodurre un client o un server di World of Warcraft.

### Riferimenti tecnici

- Repository: https://github.com/konakona185-gif/wow-friendship-crawler
- Branch principale: `main`.
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

I personaggi iniziano al livello 60 e le abilità dovranno utilizzare il grado massimo disponibile, evitando di importare inutilmente i gradi inferiori.

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

Le combinazioni razza/classe devono essere definite dal sistema di creazione del personaggio, non da una scelta indipendente del ruolo.

### Ruoli del gruppo

I ruoli del gruppo sono indipendenti dalla classe:

- Tank
- Healer
- DPS

Il Druid deve poter ricoprire tutti e tre i ruoli. Il sistema dovrà consentire anche ad altre classi di essere configurate secondo le regole e le meccaniche previste dal progetto.

### Difficoltà dei contenuti

I dungeon costituiscono il contenuto centrale del gioco. Anche i raid verranno gestiti attraverso il sistema generale dei dungeon.

Sono previste le seguenti difficoltà:

- Normal
- Heroic
- Mythic
- Mythic+
- Ascended

La difficoltà Ascended è un livello personalizzato ispirato ad Ascension e dovrà essere applicabile sia ai dungeon sia ai contenuti trattati come dungeon.

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
- Nuovi attributi, statistiche, formule e scuole di danno.

Una distinzione fondamentale: i contenuti supportati dal motore devono essere configurabili attraverso i dati; una meccanica completamente nuova richiederà invece un'implementazione specifica nel motore, da rendere successivamente riutilizzabile.

Le regole di sostituzione, estensione e precedenza tra contenuti dovranno essere definite prima di implementare un sistema di sovrascrittura completo.

### Principi di progettazione

- Evitare logiche duplicate.
- Separare dati, regole e runtime.
- Evitare sistemi troppo dipendenti dalle singole classi.
- Preferire componenti e risorse ECS riutilizzabili.
- Mantenere i sistemi estendibili per contenuti futuri.
- Non confondere i dati originali di Classic con le regole generali del motore.
- Non usare il riferimento Classic come limite tecnico per le meccaniche future.

## 4. Organizzazione attuale del codice

La struttura indicata è quella dei file effettivamente presenti nel repository al momento dell'ultimo checkpoint. Dovrà essere aggiornata dopo refactoring o cambiamenti architetturali significativi.

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

- `src/combat/aura_execution.rs`
- `src/combat/aura_runtime.rs`
- `src/combat/aura_systems.rs`
- `src/combat/custom_effects.rs`
- `src/combat/damage.rs`
- `src/combat/death.rs`
- `src/combat/displacement.rs`
- `src/combat/effect_dispatcher.rs`
- `src/combat/effect_execution.rs`
- `src/combat/effect_magnitude.rs`
- `src/combat/effect_resolution.rs`
- `src/combat/effect_runtime.rs`
- `src/combat/effect_targeting.rs`
- `src/combat/enemy_ai.rs`
- `src/combat/enemy_attacks.rs`
- `src/combat/events.rs`
- `src/combat/health.rs`
- `src/combat/healing.rs`
- `src/combat/player_attacks.rs`
- `src/combat/resources.rs`
- `src/combat/stats.rs`
- `src/combat/summons.rs`
- `src/combat/target_indicator.rs`
- `src/combat/target_selection.rs`

Sottomoduli presenti:

- `src/combat/aura_runtime/`
- `src/combat/damage/`
- `src/combat/death/`
- `src/combat/effect_dispatcher/`
- `src/combat/effect_runtime/`
- `src/combat/enemy_ai/`
- `src/combat/enemy_attacks/`
- `src/combat/player_attacks/`
- `src/combat/target_indicator/`
- `src/combat/target_selection/`

Alcuni moduli hanno già directory di test dedicate. La struttura effettiva deve essere verificata prima di spostare o rinominare file.

### Navigazione nemici

Percorso principale: `src/enemy_navigation.rs`

Moduli:

- `src/enemy_navigation/movement.rs`
- `src/enemy_navigation/movement_helpers.rs`
- `src/enemy_navigation/targets.rs`

Questo sistema gestisce la navigazione dei nemici e il movimento effettivo attraverso il sistema di movimento e collisione.

La navigazione dovrà essere estesa ai combattenti evocati quando il sistema Summon sarà integrato.

### Movimento giocatore

Percorso principale: `src/player.rs`

Moduli:

- `src/player/input.rs`
- `src/player/movement.rs`
- `src/player/movement_helpers.rs`

Il movimento dei giocatori gestisce anche le richieste di displacement attraverso il sistema fisico.

### Contenuti

Percorso principale: `src/content.rs`

Moduli attualmente presenti:

- `src/content/ability.rs`
- `src/content/aura.rs`
- `src/content/combat_attribute.rs`
- `src/content/creature.rs`
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
- Definizioni delle aure.
- Politiche configurabili di stacking, scope, refresh e scaling degli effetti periodici.
- Definizioni delle abilità.
- Definizioni degli alberi e dei talenti.
- Definizioni delle creature.
- Database centrale dei contenuti.
- Registrazione e ricerca dei contenuti.
- Validazione dei riferimenti tra contenuti.
- Caricamento RON con controllo degli errori.
- Caricamento transazionale: i dati vengono applicati al database soltanto dopo aver superato le verifiche.
- Supporto di `ContentDatabase` come `Resource` Bevy.

### Creature e catalogo per le evocazioni

È stato aggiunto il modello dati `CreatureDefinition` in `src/content/creature.rs`.

La definizione contiene i dati necessari per descrivere una creatura evocabile, tra cui:

- Identità e metadati.
- Salute.
- Danno di attacco.
- Intervallo tra gli attacchi.
- Portata corpo a corpo.
- Raggio di aggro.
- Distanza di leash.
- Velocità di movimento.

La definizione dispone di validazione e test, inclusa la serializzazione RON.

Il database centrale supporta le creature attraverso:

- `ContentDatabase.creatures`.
- Registrazione delle creature.
- Ricerca per `ContentId`.
- Conteggio delle creature registrate.
- Validazione delle definizioni.
- Controllo dei riferimenti a creature mancanti negli effetti Summon.

Il loader dei pacchetti supporta `creatures` in `ContentPackData`, con compatibilità per i vecchi pacchetti che non contengono ancora il campo.

**Importante:** il catalogo e il loader supportano le creature, ma questo non significa che il runtime di evocazione sia già completo o che il pacchetto Classic contenga già creature evocabili effettive.

### Funzionalità ancora mancanti nel sistema contenuti

- Pacchetti RON effettivi completi con abilità, effetti, aure e creature.
- Dati originali completi di abilità e talenti Classic.
- Integrazione definitiva di tutti i pacchetti con i sistemi runtime.
- Ricaricamento dinamico dei contenuti.
- Regole definitive per sovrascrittura e precedenza dei pacchetti.

La distinzione tra database registrabile e database effettivamente caricato nel runtime deve essere mantenuta: il fatto che `ContentDatabase` sia una `Resource` non significa che tutti i contenuti siano già disponibili e integrati nel gioco.

## 6. Stato attuale del sistema di combattimento

Le fondamenta del sistema di combattimento sono state consolidate e il lavoro sulla Fase 3 è focalizzato sull'esecuzione effettiva delle operazioni nel mondo ECS.

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

Per le scuole composite, la sola mitigazione più alta deve essere applicata una volta sola.

### Colpi critici

Il modulo `critical.rs` contiene il calcolo dei colpi critici.

La funzione riceve la probabilità, il moltiplicatore e il tiro già generato, determinando il risultato. La generazione casuale del tiro non è responsabilità di questo modulo.

La probabilità deve rimanere nell'intervallo 0–100 e il moltiplicatore deve essere almeno 1.

### Threat

Il sistema di minaccia utilizza valori signed.

Sono supportati incrementi e riduzioni della threat, con clamp a zero: il valore non può mai diventare negativo.

Le operazioni non valide, non finite o prive di significato vengono ignorate.

La threat deve poter essere applicata a tutti i combattenti rilevanti, non soltanto a personaggi giocanti, quando il sistema di Summon e le future entità NPC saranno pienamente integrati.

### Displacement

Il displacement dispone di una struttura runtime basata su `PendingDisplacement`.

Il sistema:

- Calcola la richiesta di spostamento.
- Accumula il displacement richiesto.
- Applica il movimento attraverso il sistema fisico.
- Rispetta la collisione e lo scorrimento.
- Mantiene le richieste quando il terreno o il sistema di collisione non sono ancora pronti.
- Protegge da valori non finiti.

Sono presenti integrazioni nel movimento del giocatore e nel movimento dei nemici.

Il displacement è considerato implementato a livello di esecuzione, ma deve continuare a essere incluso nei test end-to-end e negli audit di regressione.

## 7. Meccaniche provvisorie del combattimento

Il sistema di combattimento conserva alcune meccaniche temporanee.

### Ability 1 — Tank

- Infligge 15 danni.
- Genera 20 risorse.
- Moltiplicatore della minaccia pari a 3.

### Ability 1 — DPS

- Con almeno 30 risorse, ne consuma 30 e infligge 40 danni.
- Altrimenti infligge 10 danni e genera 15 risorse.
- Moltiplicatore della minaccia pari a 1.

### Ability 1 — Healer

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

Non applicare automaticamente nuove formule, critici o regole di mitigazione agli attacchi provvisori senza averne prima completato l'integrazione.

## 8. Stato della compilazione automatica

### Ultima revisione corrente

- Revisione: `b0b727040bb31d2352dc928701450e945b55b79d`.
- Commit: `Update combat.rs`.
- Data: 4 novembre 2026.
- Workflow: numero 446.
- Run ID: `41894159220`.
- Stato all'ultimo controllo: `in_progress`.
- Esito: non ancora disponibile.

Riferimento:
https://github.com/konakona185-gif/wow-friendship-crawler/actions/runs/41894159220

La run precedente sulla revisione `a93c7ce1939ad78575232d236b086653f3e28f5e` è stata annullata dopo il nuovo push.

Non considerare la revisione corrente superata finché il workflow associato esattamente a questo SHA non riporta `success`.

### Regole di verifica CI

- Verificare sempre la revisione esatta a cui è associato un workflow.
- Non dedurre un successo dal fatto che il workflow sia terminato.
- Non dichiarare una CI superata se lo stato è `cancelled`, `failure`, `in_progress` o `pending`.
- Non avviare inutilmente verifiche intermedie quando sono già presenti modifiche correlate da completare.
- Raggruppare le modifiche compatibili e utilizzare verifiche a blocchi quando possibile.
- Non bloccare automaticamente il lavoro in attesa di una CI verde, a meno che il risultato sia necessario per proseguire in sicurezza.
- Prima di una nuova verifica controllare se esiste già una run in corso.
- Aggiornare la roadmap quando si dispone di un nuovo esito significativo.

Una vecchia verifica positiva non certifica le revisioni successive.

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
- Modello dati delle creature e integrazione iniziale con il database.

La parte relativa alle creature è stata estesa in seguito per supportare il sistema di Summon.

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
- Gestione della threat.
- Fondamenta per le aure e gli effetti.
- Test di validazione e comportamento dei modelli.

La fase è conclusa a livello di fondamenta tecniche. L'integrazione operativa con tutte le abilità, le creature e gli attacchi definitivi viene sviluppata nelle fasi successive.

### Fase 3 — Sistema di esecuzione degli effetti

**Stato: IN CORSO**

Questa fase realizza il sistema centralizzato capace di trasformare le definizioni degli effetti in operazioni valide e di applicarle realmente alle entità del gioco.

Non dichiarare conclusa la fase sulla base della sola presenza dei modelli dati o degli handler: occorrono runtime funzionante, integrazione ECS e test adeguati.

#### 3.1 Magnitudine e risoluzione degli effetti

**Stato: implementato**

Sono presenti:

- Calcolo della magnitudine degli effetti.
- Scaling tramite attributi e coefficienti.
- Calcolo dei valori signed.
- Risoluzione delle definizioni degli effetti.
- Conversione delle definizioni in operazioni eseguibili.
- Validazione e gestione degli errori.

#### 3.2 Esecuzione degli effetti

**Stato: implementato**

Sono presenti:

- Coda delle operazioni in attesa.
- Esecutore ECS degli effetti.
- Applicazione del danno.
- Applicazione delle cure.
- Applicazione delle risorse.
- Applicazione e rimozione delle aure.
- Gestione delle operazioni che coinvolgono più componenti.
- Controlli sui dati non validi.

#### 3.3 Runtime delle aure

**Stato: implementato**

Sono presenti:

- `ActiveAura`.
- `ActiveAuras`.
- Gestione delle istanze attive.
- Durata.
- Stack.
- Refresh.
- Scadenza.
- Rimozione.
- Scope `Shared` e `PerCaster`.
- Politica di stacking configurabile.
- Refresh normale.
- Pandemic.
- Scaling degli effetti periodici in funzione degli stack.
- Test dedicati.

Le regole di comportamento devono rimanere coerenti con il riferimento progettuale Retail per quanto riguarda stacking, source scope e Pandemic, salvo modifiche esplicite alla progettazione.

#### 3.4 Effetti periodici

**Stato: implementato a livello runtime, da includere negli audit E2E**

Il sistema deve gestire:

- Intervalli periodici.
- Accumulo temporale.
- Tick.
- Effetti associati ai tick.
- Eventi `OnPeriodicTick`.
- Interazione con il dispatcher.
- Semantica di esecuzione differita.

Preservare la regola secondo cui gli effetti reattivi accodati durante un aggiornamento vengono eseguiti nell'Update successivo, non nello stesso frame.

#### 3.5 Eventi reattivi e dispatcher

**Stato: implementato**

Sono presenti:

- `CombatEventContext`.
- `CombatEventQueue`.
- `CombatEffectSubscriptions`.
- Dispatcher degli eventi reattivi.
- Gestione FIFO.
- Gestione delle generazioni.
- Limite alle generazioni di reattività per evitare cicli incontrollati.
- Trigger di scadenza e tick periodici.
- Esiti associati agli eventi delle aure.

Alla scadenza di un'aura, la sorgente dell'evento `OnExpire` deve essere il caster originale e il bersaglio deve essere il proprietario dell'aura.

Le sottoscrizioni automatiche delle abilità rimangono da integrare con il runtime definitivo delle abilità e vengono trattate nella Fase 4.

#### 3.6 Effetti AoE

**Stato: implementato nelle fondamenta runtime, da verificare nell'audit complessivo**

È presente il supporto per la risoluzione dei bersagli ad area.

Devono essere preservati:

- Raggio.
- Selezione dei bersagli.
- Relazione tra sorgente e bersaglio.
- Applicazione dell'effetto a ciascun bersaglio valido.
- Evitare duplicazioni nell'applicazione degli effetti.

L'audit finale deve verificare che tutte le categorie di bersagli previste siano coerenti con le regole di squadra e con le future entità evocate.

#### 3.7 Effetti personalizzati

**Stato: registry implementato, integrazione da verificare**

Sono presenti:

- Registry degli handler personalizzati.
- Registrazione degli handler.
- Gestione dei conflitti e degli errori.
- Interfaccia per estendere il comportamento degli effetti.

Rimane necessario verificare l'integrazione complessiva degli handler con gli effetti effettivamente caricati dai pacchetti e con il runtime.

#### 3.8 Threat ed effetti di spostamento

**Stato: implementati nelle rispettive fondamenta runtime**

Threat:

- Valori signed.
- Incrementi e riduzioni.
- Clamp a zero.
- Protezione da valori non finiti.

Displacement:

- Accodamento delle richieste.
- Risoluzione del vettore di spostamento.
- Applicazione attraverso collisioni.
- Integrazione nel movimento del giocatore e del nemico.
- Test dedicati.

L'integrazione con le entità evocate deve essere completata con il sistema Summon.

#### 3.9 Summon — catalogo e modello dati

**Stato: IMPLEMENTATO**

Sono stati aggiunti:

- `src/content/creature.rs`.
- Catalogo delle creature nel database.
- Registrazione e lookup delle creature.
- Validazione delle creature.
- Validazione dei riferimenti degli effetti Summon.
- Supporto del loader per `ContentPackData.creatures`.
- Compatibilità con i pacchetti RON legacy che non contengono il campo `creatures`.

La definizione della creatura contiene i dati di base necessari al suo runtime:

- Health.
- Attack damage.
- Attack interval.
- Melee range.
- Aggro range.
- Leash range.
- Movement speed.

#### 3.10 Summon — fondamenta runtime

**Stato: PARZIALMENTE IMPLEMENTATO**

È presente `src/combat/summons.rs`.

Sono stati introdotti:

- `SummonFaction`.
- Fazioni runtime `Allies` e `Enemies`.
- Funzione per ottenere la fazione opposta.
- `SummonedCreature`.
- Collegamento tra evocazione e owner.
- Identificativo della creatura nel catalogo.
- Memorizzazione della fazione.
- Durata opzionale.
- Tick della durata.
- Rilevamento della scadenza.
- `AllySummon`.
- Richieste `PendingSummon`.
- Risorsa `PendingSummons`.
- Coda FIFO delle richieste.
- Test per fazioni, durata e ordine delle richieste.

In `src/combat.rs` sono già stati aggiunti:

- Registrazione del modulo `summons`.
- Esportazione dei componenti pubblici.
- Inizializzazione della risorsa `PendingSummons`.

**Queste sono soltanto le fondamenta. Non esiste ancora un sistema Summon end-to-end completo.**

#### 3.11 Summon — regole di fazione obbligatorie

**Decisione progettuale già approvata dall'utente**

La fazione di un'evocazione dipende dalla fazione runtime del suo proprietario.

Regole:

- Un `Character` giocante evoca un alleato.
- Un `Enemy` ostile evoca un nemico.
- Un'evocazione alleata che a sua volta evoca un'altra creatura genera un'evocazione alleata.
- Un'evocazione nemica che genera un'altra creatura genera un'evocazione nemica.

Questa regola riguarda la squadra runtime del gioco, non la fazione razziale Alliance/Horde.

Un personaggio giocante rimane alleato ai fini del dungeon indipendentemente dalla razza scelta.

Non utilizzare `CharacterFaction` per determinare la squadra PvE delle evocazioni: rappresenta Alliance/Horde e ha una responsabilità differente.

Le evocazioni alleate sono NPC controllati dall'IA, non personaggi giocanti e non devono essere trattate come `Character`.

Le evocazioni nemiche devono essere considerate nemici a tutti gli effetti.

#### 3.12 Summon — attività ancora da completare

**Stato: DA IMPLEMENTARE**

1. Collegare l'operazione `ResolvedEffectOperation::Summon` al runtime e alla coda `PendingSummons`.
2. Recuperare sorgente, posizione, creatura, quantità e durata.
3. Determinare la fazione dell'owner con le regole approvate.
4. Implementare il sistema che consuma le richieste della coda.
5. Creare effettivamente le entità evocate.
6. Applicare le statistiche lette da `CreatureDefinition`.
7. Applicare i componenti ECS necessari al combattimento.
8. Applicare la posizione iniziale e il sistema di navigazione.
9. Integrare il timer di durata nel runtime e despawn alla scadenza.
10. Gestire la morte delle evocazioni.
11. Definire e implementare il comportamento quando l'owner viene rimosso o muore, evitando entità orfane.
12. Integrare le evocazioni alleate nell'IA e nella selezione dei bersagli.
13. Fare in modo che le evocazioni alleate individuino e attacchino i nemici.
14. Integrare le evocazioni nemiche nel comportamento dei nemici ostili.
15. Fare in modo che i nemici possano attaccare i giocatori e le evocazioni alleate.
16. Consentire al giocatore di selezionare e attaccare le evocazioni nemiche.
17. Verificare che le evocazioni alleate non vengano scambiate per nemici.
18. Verificare che le evocazioni nemiche non vengano scambiate per alleati.
19. Integrare threat e displacement.
20. Implementare test unitari e test end-to-end del ciclo di vita e del combattimento.

La soluzione dovrà evitare di duplicare completamente il sistema di combattimento dei nemici soltanto per gestire le evocazioni alleate.

È necessario valutare una struttura comune per i combattenti NPC, con fazione, selezione di bersagli, movimento, threat e attacchi.

La presenza del marcatore `Enemy` non deve essere usata in modo improprio per rappresentare un'evocazione alleata.

Il sistema deve rimanere estendibile anche a creature future, NPC, guardiani, pet e altre entità controllate dall'IA.

#### 3.13 Interrupt

**Stato: rimandato alla Fase 4**

L'interrupt runtime completo richiede l'integrazione dello stato di cast e channel delle abilità.

Non implementare una gestione parziale che non possa distinguere correttamente:

- Cast in corso.
- Channel in corso.
- Abilità istantanee.
- Cast già completati.
- Cast interrotti.

L'implementazione completa è assegnata alla Fase 4, quando verrà realizzato il runtime delle abilità.

#### 3.14 Test end-to-end e audit

**Stato: DA COMPLETARE**

Dopo aver completato il sistema Summon:

- Audit dell'intera pipeline effetti.
- Verifica dell'integrazione tra definizioni RON e runtime.
- Verifica delle interazioni tra aura, effetti, eventi, threat e displacement.
- Test delle entità evocate.
- Test delle fazioni.
- Test del ciclo di vita delle evocazioni.
- Test di morte, durata e cleanup.
- Test dell'IA e del combattimento tra fazioni.
- Test della compatibilità con entità non evocate.
- Test di regressione.
- Verifica delle CI.

#### 3.15 Contenuti RON concreti

**Stato: DA COMPLETARE**

Il pacchetto `content/packs/classic_1_12_1.ron` esiste, ma deve essere verificato e completato con definizioni effettive per le meccaniche runtime.

In particolare occorre inserire contenuti concreti per:

- Creature.
- Effetti.
- Aure.
- Abilità.
- Riferimenti tra gli elementi.

I riferimenti devono essere validi e coerenti con il database centrale.

#### Condizione per completare la Fase 3

La Fase 3 sarà conclusa soltanto dopo:

1. Implementazione end-to-end del Summon.
2. Integrazione delle evocazioni alleate e nemiche.
3. Test del ciclo di vita e del combattimento.
4. Verifica completa del runtime degli effetti.
5. Integrazione con i pacchetti di contenuti.
6. Audit e test di regressione.
7. Verifica di una CI superata sulla revisione finale.

Non utilizzare una percentuale di completamento come sostituto di questi criteri.

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
- Sottoscrizioni automatiche agli eventi reattivi.

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

Poiché i personaggi partono dal livello 60, utilizzare i gradi massimi delle abilità, salvo casi in cui un grado inferiore abbia una funzione specifica.

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
- Creature e NPC modulari.

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

Il gioco dovrà rimanere utilizzabile anche in single-player.

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
- Introdurre nuove statistiche e formule.

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

Queste regole devono essere rispettate in ogni sessione, anche quando la conversazione precedente non è disponibile.

### Workflow sui file

- Modificare un solo file alla volta.
- Prima di intervenire, leggere il contenuto completo del file dalla branch `main`.
- Non basarsi su versioni ricordate da chat precedenti.
- Dopo ogni conferma `fatto`, recuperare il file effettivo da GitHub e controllare che il commit sia presente.
- Considerare `fatto` come conferma che l'utente ha committato la modifica su `main`.
- Per modifiche strutturali o multilinea fornire sempre il file completo aggiornato.
- Quando l'utente chiede esplicitamente il file completo, fornire il file completo.
- Per una modifica di una sola riga indicare esattamente il percorso, il numero di riga e il contenuto da sostituire.
- Se bisogna aggiungere o rimuovere una riga, indicare esattamente il punto di inserimento o rimozione.
- Non usare istruzioni vaghe come "aggiungi sotto la funzione" senza un riferimento preciso.
- Quando ci sono dubbi sull'integrità di un file, preferire la consegna dell'intero file.
- Non suddividere un file completo in blocchi che possano creare ambiguità di copia.

### Architettura e refactoring

- I nomi dei file e delle directory devono descrivere chiaramente la loro responsabilità.
- Un refactoring dei nomi richiede di controllare e aggiornare tutti i moduli, gli import, i test e i riferimenti interessati.
- Evitare rinominazioni isolate che lascino collegamenti incoerenti.
- Non preoccuparsi del numero di righe durante le fasi di implementazione, salvo file che superino circa 1.500 righe.
- Dopo la stabilizzazione delle funzionalità, effettuare refactoring per ridurre la dimensione dei file e migliorare la separazione delle responsabilità.
- Non effettuare refactoring estetici mentre si sta ancora stabilizzando una meccanica, salvo necessità concreta.
- Evitare frammentazioni artificiali.

### Qualità del codice

- Non aggiungere commenti al codice.
- Non aggiungere commenti che iniziano con `//` o `#`, ad eccezione degli attributi Rust necessari come `#[derive(...)]`.
- Preservare le meccaniche già funzionanti.
- Non modificare una meccanica esistente senza spiegare prima le conseguenze e ottenere l'approvazione dell'utente.
- Preferire sistemi modulari, riutilizzabili e data-driven.
- Evitare duplicazioni di logica.
- Mantenere separati contenuti, regole di gioco e implementazione tecnica.
- Non applicare nuove formule agli attacchi provvisori prima di avere completato il relativo sistema definitivo.

### CI e commit

- Raggruppare le modifiche compatibili prima di avviare una nuova verifica automatica.
- Non avviare verifiche intermedie inutili se una run è già in corso.
- Non attendere necessariamente una CI verde per ogni singolo file quando è possibile proseguire in sicurezza.
- Non dichiarare una CI superata senza aver verificato `conclusion = success` sulla revisione esatta.
- Non confondere il risultato di una revisione precedente con quello del commit attuale.
- Dopo una modifica significativa verificare i workflow e registrare lo stato reale.

### Continuità delle sessioni

- Questo documento è la memoria tecnica primaria del progetto.
- Prima di ogni sessione verificare la situazione effettiva del repository.
- Utilizzare il documento per recuperare decisioni architetturali e attività in corso.
- Non chiedere nuovamente all'utente decisioni già registrate qui.
- Aggiornare il documento dopo cambiamenti architetturali significativi, completamento di una fase o modifiche sostanziali alla direzione del progetto.
- Non aggiornare la roadmap per ogni singolo commit.
- Quando si aggiorna la roadmap, fornire il contenuto completo per evitare sostituzioni parziali o perdita di informazioni.

### Ambito di lavoro

Il progetto principale di questa pipeline è `wow-friendship-crawler`.

Durante queste sessioni concentrarsi esclusivamente sul progetto WoW e non deviare sul progetto Godot.

## 11. Prossimi passaggi operativi

L'ordine di lavoro aggiornato al checkpoint del 4 novembre 2026 è il seguente.

### Stato iniziale

Ultimo commit verificato:

`b0b727040bb31d2352dc928701450e945b55b79d`

File aggiornati negli ultimi passaggi:

- `src/combat/summons.rs`
- `src/combat.rs`

La run CI associata al commit è ancora `in_progress`.

### Prossimo lavoro tecnico

1. Recuperare e analizzare `src/combat/effect_runtime.rs` nella sua versione completa su `main`.
2. Integrare l'operazione Summon con la coda `PendingSummons`.
3. Determinare il proprietario, la posizione iniziale, la creatura, la quantità e la durata.
4. Implementare il sistema di spawn delle evocazioni a partire dal catalogo `CreatureDefinition`.
5. Applicare statistiche e componenti runtime coerenti.
6. Integrare la fazione ereditata dall'owner.
7. Integrare navigazione, IA, acquisizione dei bersagli e combattimento per entrambe le fazioni.
8. Integrare durata, morte e cleanup.
9. Implementare i test del ciclo di vita delle evocazioni.
10. Completare l'integrazione con il pacchetto RON.
11. Eseguire l'audit generale della Fase 3.
12. Verificare i test e la CI sulla revisione finale.
13. Aggiornare la roadmap dopo il completamento effettivo della fase.

Procedere un file alla volta, controllando sempre la versione effettiva su `main`.

Non anticipare la Fase 4 e non implementare il runtime completo degli interrupt prima che sia disponibile lo stato delle abilità.

## 12. Continuità e fonte di verità

Il repository è la fonte di verità per lo stato effettivo del codice.

La roadmap si trova alla radice del repository:

`benilla-classic-pipeline.md`

Non si trova nella directory `.github`.

Questo documento serve a conservare:

- Identità e obiettivi del progetto.
- Decisioni di progettazione.
- Struttura tecnica.
- Stato effettivo delle fasi.
- Componenti già implementati.
- Componenti ancora mancanti.
- Regole operative.
- Stato della CI.
- Prossimi passaggi.

Prima di ogni intervento occorre controllare la situazione reale del repository, senza assumere che le informazioni riportate in questo documento siano necessariamente aggiornate dopo gli ultimi commit.

In caso di discrepanza tra roadmap e codice, il codice effettivamente presente su `main` è la fonte di verità per l'implementazione; le decisioni progettuali esplicitamente approvate e registrate nella roadmap rimangono invece il riferimento per il comportamento desiderato.

La roadmap deve essere aggiornata quando viene completata una fase significativa, viene introdotta una componente architetturale importante, cambia una decisione progettuale o viene modificata sostanzialmente la direzione dello sviluppo.

L'obiettivo finale è costruire un sistema di gioco modulare, coerente ed espandibile, nel quale i contenuti originali costituiscano una base solida sulla quale sviluppare liberamente nuove possibilità.
