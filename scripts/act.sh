#!/bin/bash
# test script for local workflow debugging

act --action-offline-mode -j 'build' | tee ci-build.log
act --action-offline-mode -j 'test' | tee ci-test.log
act --action-offline-mode -j 'miri' | tee ci-miri.log
act --action-offline-mode -j 'doc' | tee ci-doc.log
