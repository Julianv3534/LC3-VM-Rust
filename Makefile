LC3TOOLS_DIR := /tmp/lc3tools
ROGUE_DIR := /tmp/lc3-rogue

LC3AS := $(LC3TOOLS_DIR)/lc3as
ROGUE_SRC := $(ROGUE_DIR)/rogue.asm
ROGUE_COMPAT := $(ROGUE_DIR)/rogue_lc3as.asm
ROGUE_OBJ := $(ROGUE_DIR)/rogue_lc3as.obj

.PHONY: help rogue-setup rogue-obj rogue-run clean-rogue

help:
	@echo "Targets:"
	@echo "  make rogue-run   Build everything and run lc3-rogue on this VM"
	@echo "  make rogue-obj   Build rogue object file only"
	@echo "  make clean-rogue Remove generated compatible rogue files"

rogue-setup:
	@if [ ! -d "$(LC3TOOLS_DIR)/.git" ]; then \
		git clone https://github.com/haplesshero13/lc3tools "$(LC3TOOLS_DIR)"; \
	fi
	@if [ ! -x "$(LC3AS)" ]; then \
		cd "$(LC3TOOLS_DIR)" && \
		flex -i -Plc3 lc3.f && \
		gcc -g -Wall -o lc3as lex.lc3.c symbol.c; \
	fi
	@if [ ! -d "$(ROGUE_DIR)/.git" ]; then \
		git clone https://github.com/justinmeiners/lc3-rogue "$(ROGUE_DIR)"; \
	fi

rogue-obj: rogue-setup
	@sed \
		-e 's/xFF92/#-110/g' \
		-e 's/xFF89/#-119/g' \
		-e 's/xFF9F/#-97/g' \
		-e 's/xFF8D/#-115/g' \
		-e 's/xFF9C/#-100/g' \
		-e 's/xFFFC/#-4/g' \
		-e 's/xAC34/#-21452/g' \
		"$(ROGUE_SRC)" > "$(ROGUE_COMPAT)"
	@"$(LC3AS)" "$(ROGUE_COMPAT:.asm=)"
	@echo "Built $(ROGUE_OBJ)"

rogue-run: rogue-obj
	cargo run --release -- "$(ROGUE_OBJ)"

clean-rogue:
	rm -f "$(ROGUE_COMPAT)" "$(ROGUE_OBJ)" "$(ROGUE_DIR)/rogue.obj"
