// SPDX-License-Identifier: MIT
pragma solidity ^0.8.23;

import "forge-std/Test.sol";
import "../src/ScoringEngine.sol";

contract ScoringEngineTest is Test {
    ScoringEngineSol engine;

    function setUp() public {
        engine = new ScoringEngineSol();
    }

    function testBasicScoring() public view {
        int64[] memory factors = new int64[](3);
        factors[0] = 800;
        factors[1] = 700;
        factors[2] = 600;

        int64[] memory weights = new int64[](3);
        weights[0] = 1;
        weights[1] = 1;
        weights[2] = 1;

        int64[] memory thresholds = new int64[](3);
        thresholds[0] = 500;
        thresholds[1] = 500;
        thresholds[2] = 500;

        int64[] memory directions = new int64[](3);
        directions[0] = 1;
        directions[1] = 1;
        directions[2] = 1;

        int64[] memory bonuses = new int64[](3);
        bonuses[0] = 100;
        bonuses[1] = 100;
        bonuses[2] = 100;

        uint32[] memory corrA = new uint32[](0);
        uint32[] memory corrB = new uint32[](0);
        int64[] memory corrWeights = new int64[](0);

        int64[] memory result = engine.evaluate(
            factors, weights, thresholds, directions, bonuses,
            corrA, corrB, corrWeights, 10
        );

        assertGt(result[0], 0, "score should be > 0");
        assertLe(result[0], 1000, "score should be <= 1000");
    }

    function testRevertOnEmptyFactors() public {
        int64[] memory factors = new int64[](0);
        int64[] memory weights = new int64[](0);
        int64[] memory thresholds = new int64[](0);
        int64[] memory directions = new int64[](0);
        int64[] memory bonuses = new int64[](0);
        uint32[] memory corrA = new uint32[](0);
        uint32[] memory corrB = new uint32[](0);
        int64[] memory corrWeights = new int64[](0);

        vm.expectRevert(ScoringEngineSol.EmptyFactors.selector);
        engine.evaluate(factors, weights, thresholds, directions, bonuses, corrA, corrB, corrWeights, 10);
    }

    function testRevertOnArrayLengthMismatch() public {
        int64[] memory factors = new int64[](3);
        factors[0] = 800;
        factors[1] = 700;
        factors[2] = 600;

        int64[] memory weights = new int64[](2);
        weights[0] = 1;
        weights[1] = 1;

        int64[] memory thresholds = new int64[](3);
        thresholds[0] = 500;
        thresholds[1] = 500;
        thresholds[2] = 500;

        int64[] memory directions = new int64[](3);
        directions[0] = 1;
        directions[1] = 1;
        directions[2] = 1;

        int64[] memory bonuses = new int64[](3);
        bonuses[0] = 100;
        bonuses[1] = 100;
        bonuses[2] = 100;

        uint32[] memory corrA = new uint32[](0);
        uint32[] memory corrB = new uint32[](0);
        int64[] memory corrWeights = new int64[](0);

        vm.expectRevert(ScoringEngineSol.ArrayLengthMismatch.selector);
        engine.evaluate(factors, weights, thresholds, directions, bonuses, corrA, corrB, corrWeights, 10);
    }

    function testRevertOnCorrelationLengthMismatch() public {
        int64[] memory factors = new int64[](2);
        factors[0] = 800;
        factors[1] = 700;

        int64[] memory weights = new int64[](2);
        weights[0] = 1;
        weights[1] = 1;

        int64[] memory thresholds = new int64[](2);
        thresholds[0] = 500;
        thresholds[1] = 500;

        int64[] memory directions = new int64[](2);
        directions[0] = 1;
        directions[1] = 1;

        int64[] memory bonuses = new int64[](2);
        bonuses[0] = 100;
        bonuses[1] = 100;

        uint32[] memory corrA = new uint32[](2);
        corrA[0] = 0;
        corrA[1] = 1;

        uint32[] memory corrB = new uint32[](1);
        corrB[0] = 0;

        int64[] memory corrWeights = new int64[](2);
        corrWeights[0] = 10;
        corrWeights[1] = 10;

        vm.expectRevert(ScoringEngineSol.InvalidCorrelationLength.selector);
        engine.evaluate(factors, weights, thresholds, directions, bonuses, corrA, corrB, corrWeights, 10);
    }

    function testRevertOnInvalidCorrelationIndex() public {
        int64[] memory factors = new int64[](2);
        factors[0] = 800;
        factors[1] = 700;

        int64[] memory weights = new int64[](2);
        weights[0] = 1;
        weights[1] = 1;

        int64[] memory thresholds = new int64[](2);
        thresholds[0] = 500;
        thresholds[1] = 500;

        int64[] memory directions = new int64[](2);
        directions[0] = 1;
        directions[1] = 1;

        int64[] memory bonuses = new int64[](2);
        bonuses[0] = 100;
        bonuses[1] = 100;

        uint32[] memory corrA = new uint32[](1);
        corrA[0] = 5;

        uint32[] memory corrB = new uint32[](1);
        corrB[0] = 0;

        int64[] memory corrWeights = new int64[](1);
        corrWeights[0] = 10;

        vm.expectRevert(ScoringEngineSol.InvalidCorrelationIndex.selector);
        engine.evaluate(factors, weights, thresholds, directions, bonuses, corrA, corrB, corrWeights, 10);
    }

    function testLargeValuesDoNotRevert() public view {
        int64[] memory factors = new int64[](2);
        factors[0] = type(int64).max / 1000;
        factors[1] = type(int64).max / 1000;

        int64[] memory weights = new int64[](2);
        weights[0] = 1;
        weights[1] = 1;

        int64[] memory thresholds = new int64[](2);
        thresholds[0] = 1;
        thresholds[1] = 1;

        int64[] memory directions = new int64[](2);
        directions[0] = 1;
        directions[1] = 1;

        int64[] memory bonuses = new int64[](2);
        bonuses[0] = 100;
        bonuses[1] = 100;

        uint32[] memory corrA = new uint32[](1);
        corrA[0] = 0;

        uint32[] memory corrB = new uint32[](1);
        corrB[0] = 1;

        int64[] memory corrWeights = new int64[](1);
        corrWeights[0] = 10;

        int64[] memory result = engine.evaluate(
            factors, weights, thresholds, directions, bonuses,
            corrA, corrB, corrWeights, 10
        );

        assertGe(result[0], 0, "score should be >= 0");
        assertLe(result[0], 1000, "score should be <= 1000");
    }

    function testZeroIterations() public view {
        int64[] memory factors = new int64[](1);
        factors[0] = 500;

        int64[] memory weights = new int64[](1);
        weights[0] = 1;

        int64[] memory thresholds = new int64[](1);
        thresholds[0] = 400;

        int64[] memory directions = new int64[](1);
        directions[0] = 1;

        int64[] memory bonuses = new int64[](1);
        bonuses[0] = 100;

        uint32[] memory corrA = new uint32[](0);
        uint32[] memory corrB = new uint32[](0);
        int64[] memory corrWeights = new int64[](0);

        int64[] memory result = engine.evaluate(
            factors, weights, thresholds, directions, bonuses,
            corrA, corrB, corrWeights, 0
        );

        assertEq(result[3], 10, "iterations should default to 10");
    }

    function testAllFactorsBelowThreshold() public view {
        int64[] memory factors = new int64[](2);
        factors[0] = 100;
        factors[1] = 200;

        int64[] memory weights = new int64[](2);
        weights[0] = 1;
        weights[1] = 1;

        int64[] memory thresholds = new int64[](2);
        thresholds[0] = 500;
        thresholds[1] = 500;

        int64[] memory directions = new int64[](2);
        directions[0] = 1;
        directions[1] = 1;

        int64[] memory bonuses = new int64[](2);
        bonuses[0] = 100;
        bonuses[1] = 100;

        uint32[] memory corrA = new uint32[](0);
        uint32[] memory corrB = new uint32[](0);
        int64[] memory corrWeights = new int64[](0);

        int64[] memory result = engine.evaluate(
            factors, weights, thresholds, directions, bonuses,
            corrA, corrB, corrWeights, 10
        );

        assertEq(result[0], 0, "score should be 0 when all below threshold");
        assertEq(result[1], 0, "tier should be 0 (Low)");
    }
}
