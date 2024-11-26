# Compiler
# Compiler
CC = gcc

# Compiler flags
CFLAGS = -Wall -Wextra -g

# Executable name
TARGET = filesys

# Source directory
SRC_DIR = src

# Source file
SRC = $(SRC_DIR)/mount.c

# Build target
all: $(TARGET)

$(TARGET): $(SRC)
	$(CC) $(CFLAGS) -o $(TARGET) $(SRC)

# Clean target to remove the executable and other temporary files
clean:
	rm -f $(TARGET)

# Run the program (optional convenience target)
run: $(TARGET)
	./$(TARGET) <argument>

.PHONY: all clean run
