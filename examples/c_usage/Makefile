CC = gcc
CFLAGS = -Wall -Wextra -std=c99
LDFLAGS = -L../../target/release -lsupabase -lpthread -ldl -lm

# Detect OS for different library extensions
UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Linux)
    LDFLAGS += -Wl,-rpath,../../target/release
endif
ifeq ($(UNAME_S),Darwin)
    LDFLAGS += -Wl,-rpath,../../target/release
endif

TARGET = supabase_c_example
SOURCE = main.c

all: $(TARGET)

# Build the Rust library first
build-rust:
	cd ../.. && cargo build --release --features ffi

$(TARGET): $(SOURCE) build-rust
	$(CC) $(CFLAGS) -o $(TARGET) $(SOURCE) $(LDFLAGS)

clean:
	rm -f $(TARGET)
	cd ../.. && cargo clean

run: $(TARGET)
	LD_LIBRARY_PATH=../../target/release ./$(TARGET)

.PHONY: all clean run build-rust
