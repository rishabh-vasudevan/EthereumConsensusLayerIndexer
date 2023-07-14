#!/bin/bash

docker-compose up -d

cargo install cargo-shuttle

cargo shuttle run