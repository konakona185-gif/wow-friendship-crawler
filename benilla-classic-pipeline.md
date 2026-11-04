
# Pipeline dei contenuti e struttura del progetto

Ultimo checkpoint: 4 novembre 2026  
Branch: `main`  
Ultimo commit su `main`: `95a9a57ec2d161960a9d317cad26ee8690525851`  
Ultimo commit che contiene modifiche al codice: `b0b727040bb31d2352dc928701450e945b55b79d`  
Fase attuale: Fase 3 — Sistema di esecuzione degli effetti  
Sotto-attività attuale: completamento del sistema Summon e correzione della compilazione Rust.  
Stato CI: **failure**, errore `E0432` in `src/player/movement.rs` (dettagli e correzione nella sezione 8).

---

## 1. Identità e obiettivi del progetto

Questo repository contiene un Action RPG cooperativo in terza persona, incentrato sull'esplorazione di dungeon e sugli scontri di gruppo. Prende ispirazione da World of Warcraft Classic e da Fellowship, ma non intende riprodurre un client o un server di World of Warcraft.

### Riferimenti tecnici

- Repository: https://github.com/konakona185-gif/wow-friendship-crawler
- Linguaggio: Rust.
- Motore: Bevy 0.18.1.
- Fisica: Avian3D 0.6.
- Navigazione: Bevy Landmass.
- Base dei contenuti: World of Warcraft Classic 1.12.1, build 5875.
- Client di riferimento: Benilla.
- Revisione Benilla: `135ce8889217bf268d1dadbbabcba56df80f3108`.
- Unità di misura: 1 unità di gioco corrisponde a 1 yard di World of Warcraft.

### Fondamenti di gioco

Il personaggio parte dal livello 60. Il progetto è focalizzato sui dungeon e sulle attività cooperative: non è prevista una progressione tradizionale dal livello 1 al 60 né una campagna open world basata su missioni. Le abilità dovranno utilizzare il grado massimo disponibile, evitando di importare inutilmente i gradi inferiori.

Razze di riferimento:
- Human
- Dwarf
- Night Elf
- Gnome
- Orc
- Undead
- Tauren
- Troll

Classi di riferimento:
- Warrior
- Paladin
- Hunter
- Rogue
- Priest
- Shaman
- Mage
- Warlock
- Druid

Sono approvate alcune combinazioni personalizzate, tra cui Dwarf Shaman e Tauren Paladin. Le combinazioni razza/classe appartengono al sistema di creazione del personaggio; il ruolo non viene scelto come sostituto della classe.

Ruoli Fellowship: Tank, Healer e DPS. Il Druid deve poter ricoprire tutti e tre i ruoli.

Difficoltà previste per dungeon e contenuti raid trattati come dungeon:
- Normal
- Heroic
- Mythic
- Mythic+
- Ascended

Ascended è una difficoltà personalizzata ispirata ad Ascension, applicabile ai dungeon e ai contenuti gestiti come dungeon.

Il gioco deve poter funzionare sia in multiplayer cooperativo sia in single-player.

## 2. Principi architetturali e decisioni permanenti

L'architettura deve essere modulare, estendibile e data-driven. I contenuti vanno separati dal codice che ne implementa il comportamento.

Tecnologie per i contenuti:
- RON per le definizioni.
- Serde per serializzazione e deserializzazione.
- Rust per regole e runtime.

Classic 1.12.1 è il primo insieme di contenuti, non il limite tecnico del motore. In futuro dovrà essere possibile introdurre abilità, talenti, specializzazioni, classi, effetti, statistiche, formule, risorse, scuole di danno e contenuti personalizzati o ispirati a espansioni successive.

I contenuti già supportati dal motore devono essere configurabili tramite dati. Una meccanica completamente nuova richiede un'implementazione specifica nel motore, da rendere poi riutilizzabile.

Principi da preservare:
- Evitare logiche duplicate.
- Separare dati, regole e runtime.
- Evitare dipendenze eccessive dalle singole classi.
- Preferire componenti e risorse ECS riutilizzabili.
- Evitare frammentazioni artificiali.
- Non confondere Alliance/Horde razziale con la squadra runtime PvE.
- Non modificare meccaniche esistenti senza spiegare prima le conseguenze e ottenere l'approvazione dell'utente.

### Decisioni sul combattimento

- Le pool `HealthPool` sono la fonte autorevole della salute; `CombatStats` conserva ancora valori legacy e non deve diventare una seconda fonte di verità.
- La threat usa modifiche signed e non può scendere sotto zero.
- Gli effetti reattivi accodati durante un Update vengono eseguiti nell'Update successivo, non nello stesso frame.
- I trigger `OnExpire` devono mantenere come sorgente il caster originale e come target il proprietario dell'aura.
- Le aure devono supportare scope `Shared` e `PerCaster`, stacking configurabile, refresh normale e Pandemic.
- Pandemic conserva la durata residua entro il limite del 30% della durata base e la aggiunge alla nuova durata.
- Gli effetti periodici devono supportare scaling per stack, disattivabile oppure lineare.
- Le scuole composite applicano soltanto la mitigazione più alta tra i componenti, non la somma.
- Per combinazioni Physical + magia, confrontare le percentuali effettive di mitigazione fisica e magica.
- Interrupt runtime completo rimandato alla Fase 4, quando esisterà lo stato di cast e channel.

### Fazioni runtime e Summon

Decisione approvata: la fazione di un'evocazione deriva dalla squadra runtime del proprietario.

- `Character` giocante → `Allies`.
- `Enemy` ostile → `Enemies`.
- Evocazione alleata che evoca → evocazione alleata.
- Evocazione nemica che evoca → evocazione nemica.

La regola non dipende da `CharacterFaction` (Alliance/Horde), perché quella rappresenta la fazione razziale e non la squadra PvE del dungeon.

Le evocazioni alleate sono NPC controllati dall'IA, non `Character` giocanti. Le evocazioni nemiche sono nemici a tutti gli effetti. Un'evocazione non deve essere marcata `Enemy` soltanto per riutilizzare sistemi che attualmente selezionano i nemici: la soluzione deve modellare correttamente la fazione e i ruoli degli NPC.

Il comportamento desiderato include spawn, statistiche, durata, despawn, morte, cleanup dell'owner, navigazione, acquisizione dei bersagli, threat e combattimento. Gli alleati attaccano i nemici; i nemici attaccano i giocatori e le evocazioni alleate; i giocatori possono selezionare e attaccare le evocazioni nemiche.

## 3. Organizzazione attuale del codice

La struttura sotto è il riferimento registrato al checkpoint. Prima di spostare o rinominare file, verificare sempre il tree e il contenuto effettivo su `main`.

### Personaggi

Percorso principale: `src/character.rs`
- `src/character/build.rs`
- `src/character/class.rs`
- `src/character/race.rs`
- `src/character/tests.rs`

### Combattimento

Percorso principale: `src/combat.rs`

Moduli:
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

Sono presenti sottomoduli di test sotto `aura_runtime/`, `damage/`, `death/`, `effect_dispatcher/`, `effect_runtime/`, `enemy_ai/`, `enemy_attacks/`, `player_attacks/`, `target_indicator/` e `target_selection/`.

### Navigazione e movimento

- `src/enemy_navigation.rs`
- `src/enemy_navigation/movement.rs`
- `src/enemy_navigation/movement_helpers.rs`
- `src/enemy_navigation/targets.rs`
- `src/player.rs`
- `src/player/input.rs`
- `src/player/movement.rs`
- `src/player/movement_helpers.rs`

Il movimento dei giocatori e dei nemici gestisce le richieste di displacement attraverso il sistema fisico e le collisioni. La navigazione dovrà essere estesa agli NPC evocati.

### Contenuti

Percorso principale: `src/content.rs`

Moduli:
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

Database:
- `src/content/database/lookup.rs`
- `src/content/database/registration.rs`
- `src/content/database/validation.rs`
- `src/content/database/tests.rs`

## 4. Stato del sistema contenuti

Sono presenti identificatori con namespace, metadati, manifesti e dipendenze dei pacchetti, registro dei pacchetti, definizioni di scuole di danno, attributi, risorse, effetti, aure, abilità, talenti, database centrale, registrazione, lookup, validazione dei riferimenti e loader RON transazionale.

`ContentDatabase` è utilizzabile come `Resource` Bevy. Questo non implica che ogni pacchetto sia automaticamente caricato o che tutti i contenuti siano già collegati ai runtime.

### Creature

`src/content/creature.rs` definisce `CreatureDefinition`, con metadati e parametri runtime tra cui:
- Health.
- Attack damage.
- Attack interval.
- Melee range.
- Aggro range.
- Leash range.
- Movement speed.

La definizione è validata e dispone di test, incluso round-trip RON.

Il database supporta `creatures`, registrazione, lookup, conteggio e validazione. La validazione controlla anche i riferimenti a creature mancanti negli effetti Summon.

`ContentPackData` nel loader supporta `creatures` con default Serde, mantenendo la compatibilità con i vecchi pack che non hanno il campo.

**Non risulta ancora completato un pacchetto RON concreto con creature evocabili e riferimenti Summon effettivi.** Il catalogo e il loader non equivalgono a un runtime Summon completo.

Restano da completare nel sistema contenuti:
- Pack RON concreti completi di creature, effetti, aure e abilità.
- Contenuti originali completi di abilità e talenti Classic.
- Integrazione runtime completa dei pack.
- Ricaricamento dinamico.
- Regole di sovrascrittura e precedenza.

## 5. Stato del sistema di combattimento

### Statistiche, salute e risorse

Sono presenti:
- `CombatStats`.
- `CombatAttributes`.
- `CombatLevel`.
- `HealthPool`.
- `CombatResources`.
- `ResourcePool`.
- `HealingRequest`.
- `HealingResolution`.
- `ResourceRequest`.
- `ResourceResolution`.

Le pool supportano validazione, aggiornamento e applicazione delle operazioni. `HealthPool` è la fonte autorevole della salute; `CombatStats` conserva valori legacy e non deve essere trattato come una seconda fonte di verità.

### Scuole di danno

Scuole fondamentali:
- Physical
- Holy
- Shadow
- Nature
- Frost
- Arcane
- Fire

Sono previste scuole ibride e composite.

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

Gruppi compositi:
- Chaos
- Cosmic
- Elemental
- Blood
- Chi
- Magical

Per le scuole ibride e composite si applica soltanto la mitigazione più alta tra i componenti. Non si sommano le resistenze.

Per combinazioni Physical + magia, si confrontano le percentuali effettive di mitigazione fisica tramite armatura e mitigazione magica.

Formula dell'armatura:

`armor / (armor + 85 * attacker_level + 400)`

Mitigazione massima: 75%.

Formula della resistenza:

`min(0.75, max(resistance, 0) / (max(attacker_level, 20) * 5) * 0.75)`

La resistenza è deterministica: non vengono introdotti tiri casuali per le mitigazioni parziali.

A livello 60:
- 100 resistenza corrisponde al 25%.
- 200 resistenza corrisponde al 50%.
- 300 resistenza corrisponde al 75%.

La resistenza Holy deve essere supportata tramite un attributo dedicato `holy_resistance`.

Per le scuole composite, la mitigazione più alta viene applicata una sola volta.

### Colpi critici

Il calcolo dei colpi critici è presente nel modulo dedicato `damage/critical.rs`.

La funzione riceve probabilità, moltiplicatore e tiro già generato, determinando il risultato. La generazione casuale del tiro non è responsabilità del modulo di calcolo.

La probabilità deve rimanere nell'intervallo 0–100 e il moltiplicatore deve essere almeno 1.

### Threat

Il sistema di minaccia utilizza valori signed.

Sono supportati incrementi e riduzioni della threat, con clamp a zero. Le operazioni non valide, non finite o prive di significato vengono ignorate.

La threat dovrà essere applicabile a tutti i combattenti rilevanti, non soltanto ai personaggi giocanti, quando il sistema Summon e le future entità NPC saranno pienamente integrati.

### Displacement

È presente `PendingDisplacement`.

Il runtime accumula richieste, calcola il vettore e lo applica attraverso il movimento fisico con collisioni e scorrimento. Le richieste vengono preservate quando il terreno o il sistema di collisioni non sono pronti e i valori non finiti sono protetti. L'integrazione è presente nel movimento di giocatori e nemici; Summon dovrà essere integrato.

## 6. Meccaniche provvisorie del combattimento

Il sistema conserva alcune meccaniche temporanee, da mantenere fino alla loro sostituzione con i sistemi definitivi.

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

Queste meccaniche sono segnaposto, non rappresentano il bilanciamento definitivo e non devono essere trasformate prematuramente in regole definitive.

Non applicare automaticamente critici, mitigazioni o altre nuove formule agli attacchi provvisori senza averne prima completato l'integrazione.

## 7. Stato delle fasi

### Fase 1 — Struttura dei contenuti

**Stato: completata**

Implementati:
- Identificatori e metadati.
- Manifesti e dipendenze.
- Database centrale.
- Registrazione e lookup.
- Validazione dei riferimenti.
- Loader RON.
- Caricamento transazionale.
- Definizioni di abilità, effetti, risorse, scuole di danno, attributi, talenti e creature.
- Integrazione del catalogo creature nel database e nel loader.

### Fase 2 — Fondamenta del combattimento

**Stato: completata**

Implementati:
- Statistiche, attributi e livelli.
- Salute e risorse.
- Scuole di danno e scuole composite.
- Armatura e resistenze.
- Calcolo dei colpi critici.
- Richieste e risoluzioni del danno.
- Guarigione.
- Applicazione delle risorse.
- Contesto degli eventi.
- Threat signed con clamp a zero.
- Fondamenta del displacement.
- Test dei modelli e delle operazioni.

La fase è conclusa a livello di fondamenta tecniche. L'integrazione definitiva con tutte le abilità e gli attacchi verrà sviluppata nelle fasi successive.

### Fase 3 — Sistema di esecuzione degli effetti

**Stato: IN CORSO**

Questa fase realizza il sistema centralizzato capace di trasformare le definizioni degli effetti in operazioni valide e applicarle realmente alle entità ECS.

Non dichiarare conclusa la fase sulla sola presenza dei modelli dati o degli handler: servono runtime funzionante, integrazione ECS e test adeguati.

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
- Gestione delle operazioni su più componenti.
- Controlli sui dati non validi.

#### 3.3 Runtime delle aure

**Stato: implementato, da mantenere sotto test di regressione**

Sono presenti:
- `ActiveAura`.
- `ActiveAuras`.
- Gestione delle istanze attive.
- Durata, stack, refresh, scadenza e rimozione.
- Scope `Shared` e `PerCaster`.
- Politiche di stacking configurabili.
- Refresh normale.
- Pandemic.
- Scaling degli effetti periodici con gli stack.
- Test dedicati.

Il riferimento progettuale per buff e debuff è WoW Retail. Le regole di source scope, stacking e Pandemic devono rimanere coerenti e coperte da test.

#### 3.4 Effetti periodici ed eventi

**Stato: implementato nelle fondamenta runtime, da verificare nell'audit E2E**

Sono presenti:
- Gestione degli intervalli e tick periodici.
- Eventi `OnPeriodicTick`.
- Integrazione con dispatcher ed esecutore.
- Coda FIFO.
- Generazioni di eventi reattivi.
- Limite alle generazioni per evitare cicli incontrollati.

Gli effetti reattivi accodati durante un Update vengono eseguiti nell'Update successivo, non nello stesso frame.

Alla scadenza di un'aura, la sorgente dell'evento `OnExpire` deve essere il caster originale e il target deve essere il proprietario dell'aura.

Le sottoscrizioni automatiche delle abilità rimangono da integrare con il runtime definitivo delle abilità e sono assegnate alla Fase 4.

#### 3.5 Effetti AoE

**Stato: implementato nelle fondamenta runtime, da verificare nell'audit complessivo**

Il supporto alla risoluzione dei bersagli ad area è presente.

L'audit deve verificare:
- Raggio.
- Selezione dei bersagli.
- Relazione tra sorgente e bersaglio.
- Applicazione una sola volta a ciascun bersaglio valido.
- Coerenza con le regole di fazione.
- Compatibilità futura con le entità evocate.

#### 3.6 Effetti personalizzati

**Stato: registry implementato, integrazione da verificare**

Sono presenti:
- Registry degli handler personalizzati.
- Registrazione degli handler.
- Gestione dei conflitti e degli errori.
- Interfaccia per estendere il comportamento degli effetti.

Rimane da verificare l'integrazione degli handler con i contenuti caricati dai pacchetti e il runtime effettivo.

#### 3.7 Threat e displacement

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

L'integrazione con le entità evocate deve essere completata nel sistema Summon.

#### 3.8 Summon — catalogo e modello dati

**Stato: implementato**

Sono stati aggiunti:
- `src/content/creature.rs`.
- Catalogo delle creature nel database.
- Registrazione e lookup delle creature.
- Validazione delle definizioni.
- Validazione dei riferimenti a creature mancanti negli effetti Summon.
- Supporto del loader per `ContentPackData.creatures`.
- Compatibilità con pacchetti RON legacy senza il campo `creatures`.

La definizione della creatura contiene:
- Health.
- Attack damage.
- Attack interval.
- Melee range.
- Aggro range.
- Leash range.
- Movement speed.

#### 3.9 Summon — fondamenta runtime

**Stato: parzialmente implementato**

È presente `src/combat/summons.rs`.

Sono stati introdotti:
- `SummonFaction`.
- Fazioni runtime `Allies` ed `Enemies`.
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

In `src/combat.rs` sono stati aggiunti:
- Registrazione del modulo `summons`.
- Esportazione dei componenti pubblici.
- Inizializzazione della risorsa `PendingSummons`.

Queste sono soltanto le fondamenta. Non esiste ancora un sistema Summon end-to-end completo.

#### 3.10 Summon — attività ancora da completare

**Stato: DA IMPLEMENTARE**

1. Collegare `ResolvedEffectOperation::Summon` al runtime e alla coda `PendingSummons`.
2. Recuperare sorgente, posizione, creatura, quantità e durata.
3. Determinare la fazione dell'owner con le regole approvate.
4. Implementare il sistema che consuma le richieste della coda.
5. Creare effettivamente le entità evocate.
6. Applicare le statistiche lette da `CreatureDefinition`.
7. Applicare i componenti ECS necessari al combattimento.
8. Applicare posizione iniziale e sistema di navigazione.
9. Integrare il timer di durata e il despawn alla scadenza.
10. Gestire la morte delle evocazioni.
11. Definire e implementare il comportamento quando l'owner viene rimosso o muore, evitando entità orfane.
12. Integrare le evocazioni alleate nell'IA e nella selezione dei bersagli.
13. Fare in modo che le evocazioni alleate individuino e attacchino i nemici.
14. Integrare le evocazioni nemiche nel comportamento dei nemici ostili.
15. Fare in modo che i nemici possano attaccare giocatori ed evocazioni alleate.
16. Consentire al giocatore di selezionare e attaccare le evocazioni nemiche.
17. Verificare che le evocazioni alleate non vengano scambiate per nemici.
18. Verificare che le evocazioni nemiche non vengano scambiate per alleati.
19. Integrare threat e displacement.
20. Implementare test unitari ed end-to-end del ciclo di vita e del combattimento.

La soluzione dovrà evitare di duplicare completamente il sistema di combattimento dei nemici soltanto per gestire le evocazioni alleate.

È necessario valutare una struttura comune per i combattenti NPC, con fazione, selezione di bersagli, movimento, threat e attacchi.

La presenza del marcatore `Enemy` non deve essere usata impropriamente per rappresentare un'evocazione alleata.

Il sistema deve rimanere estendibile a creature future, NPC, guardiani, pet e altre entità controllate dall'IA.

#### 3.11 Interrupt

**Stato: rimandato alla Fase 4**

L'interrupt runtime completo richiede l'integrazione dello stato di cast e channel delle abilità.

Non implementare una gestione parziale che non distingua correttamente:
- Cast in corso.
- Channel in corso.
- Abilità istantanee.
- Cast già completati.
- Cast interrotti.

L'implementazione completa è assegnata alla Fase 4.

#### 3.12 Contenuti RON concreti

**Stato: da completare**

Il pacchetto `content/packs/classic_1_12_1.ron` deve essere verificato e completato con definizioni effettive per:
- Creature.
- Effetti.
- Aure.
- Abilità.
- Riferimenti tra gli elementi.

I riferimenti devono essere validi e coerenti con il database centrale.

#### 3.13 Test end-to-end e audit

**Stato: da completare**

Dopo aver completato Summon:
- Audit dell'intera pipeline effetti.
- Verifica dell'integrazione tra definizioni RON e runtime.
- Verifica delle interazioni tra aure, effetti, eventi, threat e displacement.
- Test delle entità evocate.
- Test delle fazioni.
- Test del ciclo di vita delle evocazioni.
- Test di morte, durata e cleanup.
- Test dell'IA e del combattimento tra fazioni.
- Test di compatibilità con entità non evocate.
- Test di regressione.
- Verifica delle CI.

#### Condizioni per completare la Fase 3

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
- Collegamento agli effetti definiti nei pacchetti.
- Sottoscrizioni automatiche agli eventi reattivi.

Le abilità dovranno essere data-driven quando supportate dal motore.

### Fase 5 — Contenuti originali Classic

**Stato: da iniziare**

Inserimento progressivo delle abilità e dei contenuti originali di Classic 1.12.1 per tutte le classi.

Dovranno essere rappresentati:
- Abilità attive e passive.
- Gradi delle abilità.
- Costi e requisiti.
- Restrizioni di classe.
- Prerequisiti.
- Effetti e interazioni.
- Scuole di danno.
- Tempi di recupero e lancio.
- Comportamenti specifici.

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
- Validazione dei requisiti.

Il sistema dovrà supportare in futuro nuovi alberi e talenti personalizzati.

### Fase 7 — Nemici e dungeon

**Stato: da iniziare**

Realizzazione di:
- Tipologie di nemici.
- Statistiche e comportamenti.
- Abilità nemiche.
- Aggro e gestione della minaccia.
- Incontri con gruppi di nemici.
- Boss e relative fasi.
- Meccaniche degli scontri.
- Progressione e difficoltà dungeon.
- Comportamenti specifici dei boss.
- Interazioni con effetti e abilità dei giocatori.
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
- Inventario.
- Influenza dell'equipaggiamento su attributi e abilità.

### Fase 9 — Sistema cooperativo

**Stato: parzialmente avviata**

Fellowship, membri, ruoli e identificazione sono presenti nelle prime strutture.

Restano da sviluppare:
- Gestione completa dei giocatori.
- Interazioni tra membri.
- Sinergie tra classi.
- Condivisione delle attività.
- Gestione dei gruppi e degli incontri.
- Sincronizzazione dello stato di gioco.
- Infrastruttura multiplayer.
- Gestione della connessione e disconnessione.
- Autorità e coerenza dello stato di combattimento.

La Fellowship non equivale a un multiplayer di rete già implementato. Il gioco dovrà rimanere utilizzabile anche in single-player.

### Fase 10 — Espansione contenuti

**Stato: pianificata**

Nuove classi, specializzazioni, abilità, talenti, scuole, risorse, effetti, statistiche e formule; set di contenuti multipli, dipendenze, precedenze e sovrascritture.

WoW Classic 1.12.1 deve rimanere il punto di partenza, non il limite del progetto.

### Fase 11 — Rifinitura e ottimizzazione

**Stato: pianificata**

Performance ECS, allocazioni, risorse, UI, feedback audio/video, bilanciamento, regressioni e rifinitura cooperativa.

## 8. CI — ultimo errore verificato e procedura di correzione

### Run 446 — primo fallimento del codice

- Run: `41894159220`
- Workflow number: `446`
- Commit: `b0b727040bb31d2352dc928701450e945b55b79d` (`Update combat.rs`)
- Esito: `failure`
- Job fallito: `Controllo preliminare Rust`
- Step fallito: `Controllo statico di tutti i target Rust`
- Step di formattazione Rust: `success`

### Run 447 — fallimento più recente

- Run: `41895763705`
- Workflow number: `447`
- Commit: `95a9a57ec2d161960a9d317cad26ee8690525851` (`Update benilla-classic-pipeline.md`)
- Esito: `failure`
- Job fallito: `Controllo preliminare Rust`
- Step fallito: `Controllo statico di tutti i target Rust`
- Step di formattazione Rust: `success`
- I job Linux e Windows di compilazione/test sono stati `skipped`, perché il controllo preliminare è fallito.

La run 447 ha ripetuto lo stesso errore della 446: il commit conteneva l'aggiornamento della roadmap, ma il codice Rust sottostante era ancora non compilabile. Il fallimento non è causato dal file Markdown.

### Errore compiler reale

```text
error[E0432]: unresolved import `super::PendingDisplacement`
  --> src/player/movement.rs:25:5
   |
25 |     PendingDisplacement,
   |     ^^^^^^^^^^^^^^^^^^^ no `PendingDisplacement` in `player`
   |
help: consider importing this struct through its public re-export instead:
      crate::combat::PendingDisplacement
```

Il compilatore conclude con `could not compile ... due to 1 previous error` ed exit code `101`.

### Diagnosi

In `src/player/movement.rs`, il blocco `use super::{ ... }` cerca `PendingDisplacement` nel modulo padre `crate::player`.

Il componente invece è definito nel sistema combat e pubblicamente riesportato da `src/combat.rs` come `crate::combat::PendingDisplacement`.

`src/player.rs` non contiene un export `PendingDisplacement`, quindi `super::PendingDisplacement` non può essere risolto. La compilazione fallisce prima dei job di test Linux e Windows.

### Correzione prevista — da applicare

File: `src/player/movement.rs`.

Nel blocco di import esistente, rimuovere `PendingDisplacement` dall'elenco `use super::{...}`. Aggiungere un import separato:

```rust
use crate::combat::PendingDisplacement;
```

Il blocco `use super` deve quindi continuare a importare da `crate::player` soltanto i componenti che appartengono effettivamente a quel modulo (`Character`, `CharacterCommand`, `GroundingState`, `MovementSpeed`, `Velocity`).

La correzione è coerente anche con il messaggio esplicito del compilatore. Non occorre spostare il componente in `player.rs` né creare un secondo componente: `crate::combat::PendingDisplacement` è già il re-export pubblico corretto.

Procedura:
1. Prima di modificare, recuperare il file completo `src/player/movement.rs` da `main`.
2. Correggere solo l'import errato in quel file.
3. Preservare il resto della logica di movimento e displacement.
4. Far verificare `cargo fmt --check` e il controllo statico Rust.
5. Dopo il commit, recuperare il file da `main` e controllare la modifica.
6. Verificare la nuova run associata allo SHA del commit corretto.
7. Considerare risolto l'errore soltanto quando il controllo preliminare passa e i job Linux/Windows vengono eseguiti; considerare CI verde solo con `conclusion = success`.

### Avvertenze sui warning

I log riportano anche warning di import non utilizzati e un `unused_mut`, ma non sono la causa del fallimento registrato. Il singolo errore che blocca la compilazione è `E0432` su `PendingDisplacement`.

Non mischiare la correzione di questo errore con un refactoring generale degli import o con la pulizia di tutti i warning. Prima ripristinare la compilazione con una modifica mirata; gli eventuali warning potranno essere valutati successivamente, senza confonderli con l'errore bloccante.

### Stato corrente CI da preservare

- Run 446: `failure`, causa `E0432` sopra descritta.
- Run 447: `failure`, stessa causa `E0432`; è la run più recente.
- La run 445 sul commit `a93c7ce1939ad78575232d236b086653f3e28f5e` è stata `cancelled` dopo il push successivo.
- La run 444 sul commit `b54e20abc4555624cc1046336c5965024a6c7d1c` è stata `cancelled`.

Non segnare nessuno di questi commit come verde. La correzione di `PendingDisplacement` è ancora da applicare e verificare.

## 9. Regole operative di sviluppo

### File e modifiche

- Modificare un solo file alla volta.
- Prima di intervenire leggere sempre il file completo dalla branch `main`.
- Non basarsi su versioni ricordate da chat precedenti.
- Dopo ogni conferma `fatto`, recuperare il file effettivo da GitHub e verificare che il commit sia presente.
- `fatto` significa che l'utente ha committato la modifica su `main`.
- Per modifiche strutturali o multilinea fornire il file completo aggiornato.
- Se l'utente chiede il file completo, fornire il file completo.
- Per una modifica di una sola riga indicare percorso, numero riga e testo esatto da sostituire.
- Per aggiungere o rimuovere righe indicare esattamente dove intervenire.
- Evitare istruzioni vaghe come "aggiungi sotto la funzione".

### Struttura e refactoring

- I nomi di file e directory devono descriverne la responsabilità.
- Rinominare un file richiede di verificare e aggiornare moduli, import, test e riferimenti.
- Durante l'implementazione non preoccuparsi del numero di righe, salvo file oltre circa 1.500 righe.
- Dopo la stabilizzazione, suddividere i file grandi dove esistono responsabilità distinte.
- Evitare refactoring estetici prematuri e frammentazioni artificiali.

### Qualità

- Non aggiungere commenti al codice (`//` o `#`), salvo attributi Rust necessari come `#[derive(...)]`.
- Preservare le meccaniche già funzionanti.
- Spiegare le conseguenze di modifiche meccaniche prima di applicarle e ottenere approvazione.
- Preferire sistemi modulari, riutilizzabili e data-driven.
- Evitare duplicazioni.
- Mantenere separati contenuti, regole e implementazione.
- Non applicare nuove formule agli attacchi provvisori prima del sistema definitivo.

### CI e commit

- Raggruppare modifiche compatibili prima di avviare nuove verifiche.
- Non avviare verifiche intermedie inutili se una run è già in corso.
- Non è necessario attendere una CI verde per ogni singolo file se è sicuro proseguire.
- Non dichiarare una CI superata senza controllare `conclusion = success` sullo SHA esatto.
- Un workflow `cancelled`, `failure`, `in_progress` o `pending` non è verde.
- Non confondere il risultato di un commit precedente con quello attuale.

### Continuità

Questo documento è la memoria tecnica primaria del progetto. Prima di ogni sessione verificare il repository e confrontare lo stato reale del codice con la roadmap. Non chiedere nuovamente decisioni già registrate. Aggiornare la roadmap dopo milestone o cambiamenti architetturali significativi, non dopo ogni commit.

Durante queste sessioni concentrarsi esclusivamente sul progetto WoW, non sul progetto Godot.

## 10. Prossimi passaggi immediati

Ordine operativo alla data del checkpoint:

1. Correggere l'import `PendingDisplacement` in `src/player/movement.rs` come descritto nella sezione 8.
2. Verificare che la compilazione preliminare Rust passi sulla nuova revisione.
3. Verificare il risultato finale della CI sulla revisione esatta; non considerarla verde prima di `success`.
4. Riprendere la Fase 3: fetch completo di `src/combat/effect_runtime.rs` da `main`.
5. Collegare `ResolvedEffectOperation::Summon` alla coda `PendingSummons`.
6. Implementare spawn, eredità della fazione, runtime delle statistiche, durata e despawn.
7. Integrare navigazione, IA, target selection, threat, attacchi e displacement per alleati e nemici.
8. Gestire morte e cleanup dell'owner.
9. Implementare test unitari e E2E Summon.
10. Completare i dati concreti RON.
11. Eseguire audit E2E della Fase 3 e regressioni.
12. Completare la Fase 3 soltanto dopo che tutti i criteri della sezione 7 sono soddisfatti.
13. Avviare la Fase 4 solo dopo la chiusura effettiva della Fase 3.

## 11. Fonte di verità e posizione della roadmap

La roadmap si trova alla radice del repository:

`benilla-classic-pipeline.md`

Non si trova in `.github`.

Il repository e il codice su `main` sono la fonte di verità per lo stato dell'implementazione. La roadmap conserva obiettivi, decisioni progettuali, stato delle fasi, problemi noti, istruzioni operative e continuità tra chat.

In caso di discrepanza:
- Il codice corrente su `main` stabilisce ciò che è effettivamente implementato.
- Le decisioni progettuali approvate e registrate qui stabiliscono il comportamento desiderato.
- I log CI stabiliscono gli errori effettivamente verificatisi.

Prima di ogni lavoro verificare commit, file interessati e CI corrente. Aggiornare questo documento quando cambia l'architettura, si risolve un problema bloccante, si completa una milestone o cambia la direzione del progetto.

L'obiettivo è costruire un sistema di gioco modulare, coerente, testabile ed espandibile, con i contenuti originali come base e senza limiti artificiali alle future estensioni.
