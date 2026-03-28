# Sicurezza

`inxi-rs` e' progettato per restare osservativo.

Questa non e' solo una promessa scritta: e' una scelta architetturale che il
codice applica davvero.

## Regole

- nessuna scrittura su filesystem
- nessuna scrittura su `/proc`, `/sys`, `/dev`
- nessuna shell
- nessuna privilege escalation
- nessun network I/O
- comandi esterni solo se auditati e in whitelist

## Root di lettura consentite

Il progetto legge solo sotto:

- `/etc`
- `/proc`
- `/sys`
- `/usr/lib`

## Comandi esterni

Se usati, devono essere:

- fissi
- auditati
- sotto path trusted
- eseguiti senza shell
- con timeout breve

## Perche' conta

Un tool di system information tende a sembrare innocuo finche' non cresce senza
disciplina.

Qui il recinto e' parte del progetto e deve restare visibile, testato e
documentato.
