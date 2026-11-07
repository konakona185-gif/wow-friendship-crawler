# Pipeline dei contenuti e struttura del progetto

Ultimo checkpoint: 7 novembre 2026  
Branch: main  
Ultimo commit di codice verificato su main: 55e001ec7dc06f5d9bbb3107f3b5ef3f96354b38 (Update main.rs)  
Fase attuale: Fase 3 completata — Sistema di esecuzione degli effetti.  
Passaggio successivo: attività preparatoria prima della Fase 4, da definire con l'utente. Non iniziare la Fase 4 automaticamente.  
Stato CI: success — Run #522 sullo SHA 55e001ec7dc06f5d9bbb3107f3b5ef3f96354b38; controllo preliminare Rust, 660 test e avvio headless superati su Linux e Windows.

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

Il movimento dei giocatori, dei nemici e delle evocazioni gestisce le richieste di displacement attraverso il sistema fisico e le collisioni. La navigazione e l'acquisizione dei bersagli includono le entità evocate; i test della Fase 3 coprono il comportamento delle evocazioni alleate e nemiche.

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

Il pacchetto `content/packs/classic_1_12_1.ron` contiene una definizione concreta della creatura Imp e i riferimenti usati dall'integrazione Summon. Il catalogo e il loader sono collegati al runtime: il flusso Summon è stato verificato end-to-end nella Fase 3.

Restano da sviluppare nelle fasi successive:
- Ampliamento dei pack RON con il catalogo completo di abilità, talenti, creature, incontri e ricompense.
- Ricaricamento dinamico dei contenuti, se richiesto dall'architettura futura.
- Regole generali di sovrascrittura e precedenza tra pacchetti.

L'integrazione runtime necessaria ai contenuti previsti dalla Fase 3 è completata e coperta da test.

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

Le scuole ibride e composite sono definite dalla composizione esplicita delle scuole fondamentali. L'ordine dei componenti non cambia la scuola: Physical + Nature e Nature + Physical identificano la stessa combinazione.

Il motore supporta in modo generico le scuole composite, ma non tutte le combinazioni elencate in questa roadmap sono già contenuti caricati nel pack Classic. Nel pacchetto attuale sono definite soltanto tre scuole composite: Shadowfrost, Entropy e Frostfire.

### Catalogo progettuale delle scuole ibride a due componenti

- `Holystrike` = Physical + Holy
- `Shadowstrike` = Physical + Shadow
- `Stormstrike` = Physical + Nature
- `Icestrike` = Physical + Frost
- `Spellstrike` = Physical + Arcane
- `Firestrike` = Physical + Fire
- `Twilight` = Holy + Shadow
- `Anima` = Holy + Nature
- `Holyfrost` = Holy + Frost
- `Divine` = Holy + Arcane
- `Radiant` = Holy + Fire
- `Plague` = Shadow + Nature
- `Shadowfrost` = Shadow + Frost
- `Curse` = Shadow + Arcane
- `Entropy` = Shadow + Fire
- `Rainstorm` = Nature + Frost
- `Astral` = Nature + Arcane
- `Firestorm` = Nature + Fire
- `Spellfrost` = Frost + Arcane
- `Frostfire` = Frost + Fire
- `Spellfire` = Arcane + Fire

La combinazione `Nature + Physical` usa lo stesso nome `Stormstrike` della combinazione `Physical + Nature`: l'ordine dei componenti non cambia la scuola risultante.

### Esempi progettuali di gruppi compositi

- `Chaos` = Physical + Holy + Shadow + Nature + Frost + Arcane + Fire (tutte le scuole fondamentali)
- `Cosmic` = Arcane + Holy + Nature + Shadow
- `Elemental` = Fire + Frost + Nature
- `Blood` = Physical + Nature + Shadow
- `Chi` = Nature + Holy + Shadow
- `Magical` = Arcane + Frost + Fire

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

Il sistema Summon è integrato con le entità NPC previste dalla Fase 3. Le future tipologie di combattenti dovranno riutilizzare la gestione della threat senza introdurre fonti di verità o sistemi duplicati.

### Displacement

È presente `PendingDisplacement`.

Il runtime accumula richieste, calcola il vettore e lo applica attraverso il movimento fisico con collisioni e scorrimento. Le richieste vengono preservate quando il terreno o il sistema di collisioni non sono pronti e i valori non finiti sono protetti. L'integrazione è presente nel movimento di giocatori, nemici ed entità evocate.

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

**Stato: COMPLETATA — 7 novembre 2026**

La fase ha realizzato il sistema centralizzato capace di trasformare le definizioni degli effetti in operazioni valide e applicarle realmente alle entità ECS.

Runtime, integrazione ECS, contenuti RON, Summon, audit e test richiesti dalla fase sono stati completati. La CI finale è verde sulla revisione `55e001ec7dc06f5d9bbb3107f3b5ef3f96354b38` (Run #522).

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

**Stato: implementato e verificato**

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

**Stato: implementato e verificato end-to-end**

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

**Stato: implementato e verificato**

Il supporto alla risoluzione dei bersagli ad area è presente.

Sono stati verificati:
- Validazione del raggio e del centro.
- Selezione dei bersagli in base alla relazione tra sorgente e bersaglio.
- Esclusione di entità non valide e bersagli morti.
- Applicazione una sola volta a ciascun bersaglio valido.
- Coerenza con le regole di fazione, incluse le evocazioni alleate.
- Scaling degli effetti periodici AoE in base agli stack.

#### 3.6 Effetti personalizzati

**Stato: implementato e verificato end-to-end**

Sono presenti:
- Registry degli handler personalizzati.
- Registrazione e rimozione degli handler.
- Gestione dei conflitti e degli errori.
- Esecuzione degli handler nel contesto runtime degli effetti.
- Comportamento definito quando un handler non è registrato.
- Test degli handler registrati e mancanti.
- Test di caricamento RON di un effetto `Custom`, risoluzione nel database e successiva esecuzione nel runtime.

#### 3.7 Threat e displacement

**Stato: implementati e verificati nel runtime della Fase 3**

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

L'integrazione necessaria con le entità evocate è stata completata nel sistema Summon.

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

**Stato: implementato**

È presente `src/combat/summons.rs`.

Sono implementati:
- `SummonFaction` e fazioni runtime `Allies` ed `Enemies`.
- Derivazione della fazione runtime dalla squadra del proprietario.
- `SummonedCreature` e collegamento tra evocazione, owner e creatura del catalogo.
- Durata opzionale, tick e rilevamento della scadenza.
- `AllySummon` per distinguere le evocazioni alleate dai personaggi giocanti e dai nemici.
- Richieste `PendingSummon` e coda FIFO `PendingSummons`.
- Registrazione dei componenti e delle risorse necessari.
- Test dedicati a fazioni, durata e ordine delle richieste.

#### 3.10 Summon — integrazione end-to-end

**Stato: implementato e verificato**

Il flusso Summon è collegato al runtime degli effetti e verificato con test di integrazione. Sono coperti:
1. Risoluzione di `ResolvedEffectOperation::Summon` e accodamento delle richieste.
2. Recupero di owner, creatura, quantità e durata.
3. Derivazione della fazione runtime dalla squadra dell'owner, senza confondere Alliance/Horde con la squadra PvE.
4. Consumo della coda e creazione delle entità evocate.
5. Applicazione delle statistiche definite in `CreatureDefinition`.
6. Componenti ECS per distinguere evocazioni alleate e nemiche.
7. Durata, scadenza e despawn.
8. Gestione del ciclo di vita, inclusi morte e cleanup.
9. Integrazione con navigazione e acquisizione dei bersagli.
10. Comportamento dell'IA delle evocazioni alleate contro i nemici.
11. Integrazione delle evocazioni nemiche nel comportamento ostile.
12. Selezione e combattimento coerenti con le fazioni.
13. Test unitari ed end-to-end del ciclo di vita e del combattimento, inclusi i riferimenti ai contenuti RON.

Le evocazioni alleate sono NPC controllati dall'IA e non vengono rappresentate come `Character` o marcate impropriamente come `Enemy`. Le evocazioni nemiche partecipano al combattimento come entità ostili.

Il sistema rimane estendibile a creature, pet, guardiani e altre entità NPC senza duplicare integralmente il sistema di combattimento.

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

**Stato: integrati e verificati per lo scope della Fase 3**

Il pacchetto `content/packs/classic_1_12_1.ron` include contenuti concreti verificati dal loader e dai test:
- Creatura Imp.
- Effetti e riferimenti agli effetti.
- Aura e configurazione periodica di Corruption.
- Abilità Corruption e relativo collegamento agli effetti.
- Scuole di danno fondamentali.
- Tre scuole composite attualmente definite: Shadowfrost, Entropy e Frostfire.
- Riferimenti usati dall'integrazione Summon.

I test end-to-end verificano il caricamento dei contenuti RON e l'integrazione dei riferimenti con i runtime previsti dalla Fase 3, inclusi Corruption, Custom Effects e Summon.

Questo completamento riguarda l'infrastruttura e il contenuto dimostrativo necessari alla Fase 3; non equivale all'importazione completa di tutte le abilità Classic, che appartiene alla Fase 5.

#### 3.13 Test end-to-end e audit

**Stato: completato**

È stato eseguito l'audit complessivo della pipeline degli effetti e delle relative integrazioni. La copertura include:
- Risoluzione e dispatch degli effetti.
- Aure, stack, scope per caster, refresh e Pandemic.
- Effetti periodici, timer indipendenti e comportamento alla scadenza.
- Corruption caricata dal pack Classic e verificata end-to-end.
- Eventi reattivi e trigger `OnExpire`.
- Effetti AoE, relazioni di fazione e selezione dei bersagli.
- Custom Effects registrati, mancanti e caricati da RON.
- Threat e displacement.
- Summon alleati e nemici, IA, navigazione, durata, morte e cleanup.
- Scuole composite e mitigazione della componente più efficace senza sommare le resistenze.
- Validazione del database e test di regressione.

La revisione finale è stata verificata con la Run #522: 660 test superati e 0 falliti sia su Linux sia su Windows; controllo preliminare Rust e avvio headless superati su entrambe le piattaforme.

#### Condizioni per completare la Fase 3

**Tutte le condizioni sono soddisfatte:**
1. Implementazione end-to-end del Summon.
2. Integrazione delle evocazioni alleate e nemiche.
3. Test del ciclo di vita e del combattimento.
4. Verifica del runtime degli effetti previsto dalla Fase 3.
5. Integrazione con i pacchetti di contenuti RON.
6. Audit e test di regressione.
7. CI superata sulla revisione finale: Run #522, SHA `55e001ec7dc06f5d9bbb3107f3b5ef3f96354b38`.

La Fase 3 è completata. Interrupt completo, cast e channel rimangono correttamente assegnati alla Fase 4.

### Fase 4 — Sistema di esecuzione delle abilità

**Stato: da iniziare — in attesa del passaggio preparatorio concordato con l'utente**

Non iniziare la Fase 4 automaticamente. Prima l'utente deve descrivere e affrontare un passaggio preparatorio importante previsto dopo la Fase 3. Registrare le decisioni e gli eventuali cambiamenti alla roadmap prima di iniziare la Fase 4.

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

## 8. CI — correzione `PendingDisplacement` e stato verificato

La Run #450 documentata in questa sezione è uno storico checkpoint intermedio della Fase 3. La CI finale della Fase 3 è la Run #522, registrata nella sezione 8.1.

### Errore compiler originario

Le Run 446, 447 e 448 hanno registrato il fallimento del controllo preliminare Rust a causa dell'import non risolto `super::PendingDisplacement` in `src/player/movement.rs`.

```text
error[E0432]: unresolved import `super::PendingDisplacement`
  --> src/player/movement.rs
help: consider importing this struct through its public re-export:
      crate::combat::PendingDisplacement
```

### Diagnosi

Il blocco `use super::{ ... }` cercava `PendingDisplacement` nel modulo padre `crate::player`. Il componente appartiene invece al sistema combat ed è esposto come `crate::combat::PendingDisplacement`.

### Correzione applicata

Commit corretto: `aabedbb03cc071857a19370006e6c3bff5ebf734` (`Update movement.rs`).

In `src/player/movement.rs`:
- `PendingDisplacement` è stato rimosso dall'elenco `use super::{...}`.
- È stato aggiunto l'import indipendente `use crate::combat::PendingDisplacement;`.
- La logica di movimento e displacement è rimasta invariata.

Il commit intermedio `6d4739ac65b4ee76d8e72c5a12a74e1d39dd6b0b` aveva collocato l'import all'interno del blocco `use super` e non era corretto. La Run 449 associata a quel commit è stata `cancelled`. La correzione effettiva è quella del commit `aabedbb...`.

### Run 450 — CI superata

- Run ID: `42356059517`.
- Workflow number: `450`.
- Commit: `aabedbb03cc071857a19370006e6c3bff5ebf734`.
- Esito finale: **success**.
- `Controllo preliminare Rust`: `success`, inclusi formattazione e controllo statico di tutti i target Rust.
- `Compilazione e test Linux`: `success`, inclusi compilazione, test unitari e test headless.
- `Compilazione e test Windows`: `success`, inclusi compilazione, test unitari e test headless.

La correzione di `PendingDisplacement` è verificata e l'errore `E0432` è risolto sullo SHA indicato. La CI non è più un blocco per la prosecuzione della Fase 3.

Run: https://github.com/konakona185-gif/wow-friendship-crawler/actions/runs/42356059517

### Storico delle run precedenti

- Run 446 (`41894159220`): `failure`, errore `E0432`.
- Run 447 (`41895763705`): `failure`, stesso errore.
- Run 448 (`41899879287`): `failure`, prima della correzione effettiva.
- Run 449 (`42355965898`): `cancelled`, commit intermedio con import collocato nel blocco errato.
- Run 450 (`42356059517`): `success`, correzione verificata su Linux e Windows.

Considerare una CI verde soltanto quando `conclusion = success` sulla run associata allo SHA esatto. Un workflow `cancelled`, `failure`, `in_progress` o `pending` non è verde.

## 8.1 CI finale della Fase 3 — Run #522

- Run ID: `42401651307`.
- Workflow number: `522`.
- Commit: `55e001ec7dc06f5d9bbb3107f3b5ef3f96354b38` (`Update main.rs`).
- Branch: `main`.
- Esito finale: **success**.
- `Controllo preliminare Rust`: `success`.
- `Compilazione e test Linux`: `success`; 660 test superati, 0 falliti; test headless superati.
- `Compilazione e test Windows`: `success`; 660 test superati, 0 falliti; test headless superati.

Lo SHA della Run #522 coincide con la revisione finale verificata su `main`. Questa è la CI usata per dichiarare completata la Fase 3.

Run: https://github.com/konakona185-gif/wow-friendship-crawler/actions/runs/42401651307

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

- I nomi di file e directory devono descrivere la responsabilità.
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

La Fase 3 è completata e verificata dalla Run #522.

Prima di iniziare la Fase 4 è previsto un passaggio preparatorio importante che sarà specificato dall'utente nella prossima sessione.

Istruzioni operative:
1. Non iniziare automaticamente la Fase 4.
2. Ripartire dalla Fase 3 completata e dalla CI verde Run #522 sullo SHA `55e001ec7dc06f5d9bbb3107f3b5ef3f96354b38`.
3. Lasciare che l'utente descriva il passaggio preparatorio previsto prima della Fase 4.
4. Aggiornare questa roadmap con le decisioni e le attività di quel passaggio quando saranno definite.
5. Avviare la Fase 4 soltanto dopo aver completato il passaggio preparatorio approvato.

Non riaprire l'audit della Fase 3 senza un errore o una regressione concreta.

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
