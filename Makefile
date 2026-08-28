CC     ?= cc
CFLAGS ?= -std=c11 -Wall -Wextra -Wpedantic -Iinclude
PREFIX ?= /usr/local
HDRS   := $(wildcard include/sdm/*.h)

.PHONY: check install uninstall

# Header-only: nothing to build, just make sure every header compiles alone.
check:
	@for h in $(HDRS); do echo "$$h" && $(CC) $(CFLAGS) -fsyntax-only -x c $$h || exit 1; done

install:
	@for h in $(HDRS); do install -Dm644 $$h $(DESTDIR)$(PREFIX)/$$h; done

uninstall:
	rm -f $(DESTDIR)$(PREFIX)/include/sdm/*.h
