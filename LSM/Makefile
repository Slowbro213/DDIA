CC      := gcc
CFLAGS  := -Wall -Wextra -O3 -Ilib -Isrc
LDFLAGS :=

SRC_DIR   := src
TEST_DIR  := tests
OUT_DIR   := out
BIN_DIR   := bin

APP_TARGET  := $(BIN_DIR)/app
TEST_TARGET := $(BIN_DIR)/test

APP_SRC := $(wildcard $(SRC_DIR)/*.c)

TEST_SRC := $(wildcard $(TEST_DIR)/*.c) $(filter-out $(SRC_DIR)/main.c,$(APP_SRC))

APP_OBJ  := $(patsubst %.c,$(OUT_DIR)/%.o,$(APP_SRC))
TEST_OBJ := $(patsubst %.c,$(OUT_DIR)/%.o,$(TEST_SRC))

.PHONY: all clean run test dirs

all: dirs $(APP_TARGET)

dirs:
	mkdir -p $(OUT_DIR)/$(SRC_DIR) $(OUT_DIR)/$(TEST_DIR) $(BIN_DIR)

$(APP_TARGET): $(APP_OBJ)
	$(CC) $(CFLAGS) $^ -o $@ $(LDFLAGS)

$(TEST_TARGET): $(TEST_OBJ)
	$(CC) $(CFLAGS) $^ -o $@ $(LDFLAGS)

$(OUT_DIR)/%.o: %.c
	@mkdir -p $(dir $@)
	$(CC) $(CFLAGS) -c $< -o $@

run: all
	./$(APP_TARGET)

test: dirs $(TEST_TARGET)
	./$(TEST_TARGET)

clean:
	rm -rf $(OUT_DIR) $(BIN_DIR)

