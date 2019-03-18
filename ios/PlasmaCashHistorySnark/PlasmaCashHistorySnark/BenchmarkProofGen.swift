//
//  BenchmarkProofGen.swift
//  PlasmaCashHistorySnark
//
//  Created by Anton on 14/03/2019.
//  Copyright © 2019 Home. All rights reserved.
//

import Foundation

class BenchmarkProofGen {
    func test(treeDepth: UInt32,
              numberOfBlocks: UInt32,
              nonInclusionLevel: UInt32) {
        test_benchmark_proof_gen_for_ios(treeDepth,
                                         numberOfBlocks,
                                         nonInclusionLevel)
    }
}
