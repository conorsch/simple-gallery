# simple-gallery changelog

## 0.1.0
This release caves on the no-js policy, adding a wee bit of vanilla js
to keep things sane. In the process, nice-to-haves like fade transitions
are now broken, but the upside is that the program scales well now:
it no longer attempts to load all pictures into the DOM with different
z indices, which was untenable on large (>100) image directories.

* uses minimal javascript for transitions
* fix fullpath `--directory` handling via /static route
* fix: `--bind-address` now takes a socketaddr

## 0.0.2
* feat: add /random route
* feat: add --file-extension flag
* chore: bump axum 0.5 -> 0.7

## 0.0.1
* first version
