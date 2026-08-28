CC     ?= cc
AR     ?= ar
CFLAGS ?= -std=c11 -Wall -Wextra -Wpedantic -Os -Iinclude
PREFIX ?= /usr/local

HDRS := $(wildcard include/sdm/*.h)
SRCS := $(wildcard src/*.c)
OBJS := $(SRCS:src/%.c=build/%.o)

.PHONY: all check clean install uninstall

all: build/libsdm.a

build/libsdm.a: $(OBJS)
	$(AR) rcs $@ $^

build/%.o: src/%.c
	@mkdir -p build
	$(CC) $(CFLAGS) -c -o $@ $<

# every header must also compile standalone
check:
	@for h in $(HDRS); do echo "$$h" && $(CC) $(CFLAGS) -fsyntax-only -x c $$h || exit 1; done

clean:
	rm -rf build

install: all
	@for h in $(HDRS); do install -Dm644 $$h $(DESTDIR)$(PREFIX)/$$h; done
	install -Dm644 build/libsdm.a $(DESTDIR)$(PREFIX)/lib/libsdm.a

uninstall:
	rm -f $(DESTDIR)$(PREFIX)/include/sdm/*.h $(DESTDIR)$(PREFIX)/lib/libsdm.a
