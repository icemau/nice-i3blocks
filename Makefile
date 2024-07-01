CC = gcc
CFLAGS = -Wall -Werror -g
INSTALL_DIR ?= ./out

SRC = $(wildcard *.c)

BIN = $(SRC:.c=)

all: $(BIN)

%: %.c
	$(CC) $(CFLAGS) -o $@ $<

clean:
	rm -f $(INSTALL_DIR)
