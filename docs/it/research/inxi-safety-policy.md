# Safety Policy del Clone Rust

## Obiettivo

Il clone deve essere strettamente osservativo.
Non deve modificare il sistema locale.

## Regole fissate

- niente scritture su filesystem
- niente scritture su `/proc`, `/sys`, `/dev`
- niente shell command (`sh -c`, `bash -c`, simili)
- niente network I/O
- niente privilege escalation
- niente esecuzione di binari trovati in path non trusted
- niente comandi esterni arbitrari

## Accesso filesystem consentito

Solo lettura, solo su root esplicitamente consentite:

- `/etc`
- `/proc`
- `/sys`
- `/usr/lib`

Questo serve anche a bloccare aperture accidentali di device node come `/dev/*`.

## Comandi esterni

Se e quando verranno usati, dovranno rispettare tutti questi vincoli:

- essere in whitelist
- avere argomenti fissi auditati
- essere risolti solo sotto:
  - `/usr/bin`
  - `/usr/sbin`
  - `/bin`
  - `/sbin`
- essere eseguiti senza shell
- avere `stdin` chiuso
- avere timeout breve
- non richiedere `sudo`

## Stato attuale del codice

Il bootstrap attuale legge solo dati da:

- `/etc/os-release`
- `/proc/*`
- `/sys/class/dmi/id/*`
- environment variables

Il progetto non scrive nulla sul laptop.
