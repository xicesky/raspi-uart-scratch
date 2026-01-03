
#######################################################
# Configure this.
################################################# # # #

# Compiler to use for cross-compiling
CC=arm-linux-gnueabihf-gcc


# Targets (.c files have to exist)
OBJS = helloworld


# Libraries
#LIBS := mingw32 SDLmain SDL


# Machine dep. options
GCC_MACHINEFLAGS=


# C flags
CFLAGS=-g -std=c99 -Wall
#CFLAGS=-g -std=c99 -Wall -I./include

CPPFLAGS=
CXXFLAGS=$(CPPFLAGS)

#LDFLAGS=-L./lib
#ASFLAGS=

#######################################################
# Don't change this
################################################# # # #

LIBFLAGS := $(addprefix -l,$(LIBS))
OBJFILES := $(patsubst %,target/obj/%.o,$(OBJS))


#######################################################
# Targets
################################################# # # #

.PHONY: default test runmain clean

default: target/main

target:
	mkdir target

target/obj: | target
	mkdir target/obj

target/obj/%.o: c-utils/%.c | target/obj
	$(CC) $(GCC_MACHINEFLAGS) $(CFLAGS) -c -o $@ $<

test:
	echo CFLAGS=\"$(CFLAGS)\"

target/main: $(OBJFILES) | target
#	echo LINKING: $^
	$(CC) $(GCC_MACHINEFLAGS) $(CFLAGS) $(LDFLAGS) -o $@ $^ $(LIBFLAGS)

runmain: target/main
	./target/main

clean:
	-@rm $(OBJFILES) >/dev/null 2>&1
	-@rm target/* >/dev/null 2>&1
