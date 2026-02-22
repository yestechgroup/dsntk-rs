#!/usr/bin/env bash

# Test script for NZ Probation Trial DMN Table
# This script demonstrates how to test the decision table with different scenarios

echo "Testing NZ Probation Trial DMN Table with different scenarios"
echo "============================================================="
echo ""

# Test 1: Compliant probation case (should return Compliant/Low)
echo "Test 1: Compliant Probation Case"
echo "---------------------------------"
dsntk edt --markdown nz-probation-test-1-compliant.ctx nz-probation-trial-dmn-table.md | jq
echo ""

# Test 2: Non-compliant trial case (should return Non-Compliant/High)
echo "Test 2: Non-Compliant Trial Case (AEWV visa)"
echo "---------------------------------------------"
dsntk edt --markdown nz-probation-test-2-non-compliant.ctx nz-probation-trial-dmn-table.md | jq
echo ""

# Test 3: Requires Review case (should return Requires Review/Medium)
echo "Test 3: Requires Review Case (missing notice period)"
echo "-----------------------------------------------------"
dsntk edt --markdown nz-probation-test-3-requires-review.ctx nz-probation-trial-dmn-table.md | jq