#!/bin/bash

args=$*
sh ./scripts/build_back.sh $args
sh ./scripts/build_front.sh $args
