# This makefile is to build a linux binary on macos until I find the built-in
# cross compilation tools easier.

CURRENT_USER := $(shell id -u)
CURRENT_GROUP := $(shell id -g)
PWD := $(shell pwd)
DOCKER_IMAGE_NAME = rusty-butler-builder

linux_release: build_release
	@echo "Binary is in target/release/"

build_image:
	docker build -t $(DOCKER_IMAGE_NAME) .

build_release: build_image
	docker run --rm --user "$(CURRENT_USER)":"$(CURRENT_GROUP)" -v "$(PWD)":/rusty-butler -w /rusty-butler $(DOCKER_IMAGE_NAME):latest cargo build --release
