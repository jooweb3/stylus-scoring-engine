// SPDX-License-Identifier: MIT
pragma solidity ^0.8.23;

contract ScoringEngineSol {
    function evaluate(
        int64[] calldata factors,
        int64[] calldata weights,
        int64[] calldata thresholds,
        int64[] calldata directions,
        int64[] calldata bonuses,
        uint32[] calldata corrA,
        uint32[] calldata corrB,
        int64[] calldata corrWeights,
        uint32 iterations
    ) external pure returns (int64[] memory) {
        if (iterations == 0) iterations = 10;
        uint256 numRules = weights.length;
        uint256 numFactors = factors.length;
        uint256 numCorr = corrA.length;

        int256 finalScore = 0;

        // iterative scoring with convergence
        for (uint32 iter = 0; iter < iterations; iter++) {
            int256 rawScore = 0;
            int256 totalWeight = 0;
            int256 bias = iter == 0 ? int256(0) : finalScore / 1000;

            for (uint256 i = 0; i < numRules; i++) {
                if (i >= numFactors) continue;

                int256 weight = int256(weights[i]);
                totalWeight += weight;
                int256 value = int256(factors[i]);
                int256 threshold = int256(thresholds[i]);

                bool passes;
                if (directions[i] == 1) {
                    passes = value >= threshold;
                } else {
                    passes = value <= threshold;
                }

                if (passes) {
                    int256 bonus = int256(bonuses[i]);
                    int256 adjusted = bonus + (bias * 5);
                    int256 weighted = adjusted * weight;
                    rawScore += weighted;
                }
            }

            if (totalWeight > 0) {
                int256 maxPossible = totalWeight * 100;
                finalScore = (rawScore * 1000) / maxPossible;
            }
        }

        // cross-factor correlation scoring with damped oscillation
        int256 correlationAdj = 0;
        for (uint256 i = 0; i < numCorr; i++) {
            uint256 idxA = uint256(corrA[i]);
            uint256 idxB = uint256(corrB[i]);
            if (idxA >= numFactors || idxB >= numFactors) continue;

            int256 valA = int256(factors[idxA]);
            int256 valB = int256(factors[idxB]);
            int256 pairWeight = int256(corrWeights[i]);

            // normalized product interaction
            int256 interaction = (valA * valB * pairWeight) / (int256(100000) * int256(100000));

            // damped oscillation series
            int256 damped = interaction;
            for (uint256 k = 1; k <= 50; k++) {
                int256 decay = int256(1e18) / int256(k);
                damped = (damped * decay) / int256(1e18) + (interaction * decay) / int256(1e18);
            }

            correlationAdj += damped;
        }

        int256 adjusted = finalScore + (correlationAdj * 100);
        if (adjusted < 0) adjusted = 0;
        if (adjusted > 1000) adjusted = 1000;

        int64 score = int64(adjusted);
        int64 tier;
        if (score <= 250) tier = 0;
        else if (score <= 500) tier = 1;
        else if (score <= 750) tier = 2;
        else tier = 3;

        int64[] memory result = new int64[](4);
        result[0] = score;
        result[1] = tier;
        result[2] = int64(correlationAdj * 100);
        result[3] = int64(uint64(iterations));
        return result;
    }
}
