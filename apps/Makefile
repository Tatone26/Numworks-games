GAMES := snake connectfour solitaire tetris flappybird alltheapps

ROOT_DIR := $(dir $(abspath $(lastword $(MAKEFILE_LIST))))

EXEC_PATH := target/thumbv7em-none-eabihf/release


build:
	$(foreach dir, $(GAMES), \
		cd $(ROOT_DIR)$(dir); \
		cargo build --release; \
		touch $(ROOT_DIR)$(dir).nwa; \
		cp -v $(ROOT_DIR)$(dir)/$(EXEC_PATH)/$(dir) $(ROOT_DIR)$(dir).nwa; \
		)

clean:
	$(foreach dir, $(GAMES), \
		rm -f $(ROOT_DIR)$(dir).nwa; \
		cd $(ROOT_DIR)$(dir); \
		cargo clean;)
