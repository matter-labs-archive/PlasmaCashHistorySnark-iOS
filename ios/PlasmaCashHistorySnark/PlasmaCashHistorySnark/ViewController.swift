//
//  ViewController.swift
//  PlasmaCashHistorySnark
//
//  Created by Anton Grigorev on 21/01/2019.
//  Copyright © 2019 Home. All rights reserved.
//

import UIKit

class ViewController: UIViewController {

    override func viewDidLoad() {
        super.viewDidLoad()
        NonInclusion().test()
        print("Non inclusion tested \n--------------------\n")
        PedersenHasher().test()
        print("Pedersen hasher tested \n--------------------\n")
        BenchmarkProofGen().test(treeDepth: 24, numberOfBlocks: 1, nonInclusionLevel: 2)
        print("Tests finished")
    }

}

