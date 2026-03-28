# Panoramica

`inxi-rs` non nasce per copiare riga per riga `inxi`.

Nasce per costruire un clone Rust serio, leggibile e mantenibile, che prenda le
idee migliori del tool originale e le porti in una architettura piu' moderna.

## Obiettivo

L'obiettivo reale e':

- mantenere un tool utile per supporto e diagnostica
- separare raccolta dati e presentazione
- avere un modello dati stabile
- mantenere una forte attenzione alla safety
- preparare il terreno per una futura TUI

## Stato attuale

Il progetto supporta gia':

- Linux
- output `screen`
- output `json`
- modalita' `--self-check`
- filtri privacy
- sezione `System`
- sezione `Machine`
- sezione `CPU`
- sezione `Memory`
- sezione `Graphics`
- sezione `Network`
- sezione `Drives`
- sezione `Partitions`
- sezione `Swap`
- sezione `Info`

## Identita' del progetto

`inxi-rs` e':

- liberamente ispirato a `inxi`
- una reimplementazione indipendente
- non un port ufficiale
- non un progetto affiliato all'upstream

Questa distinzione e' importante sia tecnicamente sia comunicativamente.
