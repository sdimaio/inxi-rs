# Roadmap

Il prossimo obiettivo non e' aggiungere sezioni a caso.

Il prossimo obiettivo e' arrivare a una prima release stabile credibile.

## Prima della stabile

Servono soprattutto questi blocchi:

1. confronto multi-host con `inxi`
2. ulteriore hardening di warning e fallback
3. congelamento del contratto CLI
4. congelamento del contratto JSON
5. rifinitura documentazione

## Dopo la stabile

Le estensioni piu' naturali sono:

- metadata grafici piu' ricchi
- riepiloghi storage piu' ricchi
- `Audio`
- `Battery`
- `USB`
- `Sensors`
- backend BSD
- frontend TUI

## Direzione TUI

La TUI non e' un extra cosmetico.

E' un frontend futuro previsto fin dall'inizio. Per questo il progetto e' stato
strutturato con:

- request neutre rispetto al frontend
- collector separati
- modello dati strutturato
- diagnostica `self-check`
