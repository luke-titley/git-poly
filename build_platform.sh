#!/usr/bin/env bash

PLATFORM=$1

mkdir -p ${PLATFORM}/target
rm -rf ${PLATFORM}/target

# Do the build
docker build -t git-poly:${PLATFORM} -f ${PLATFORM}/Dockerfile .
docker create --name cont1 git-poly:${PLATFORM}
docker cp cont1:/target ${PLATFORM}/target
docker rm cont1
