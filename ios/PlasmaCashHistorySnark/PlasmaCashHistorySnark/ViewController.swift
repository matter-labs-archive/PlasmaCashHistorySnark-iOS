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
        Circuit().test()
        print("Circuit tested")
        SparseMerkleTree().test()
        print("Sparse merkle tree tested")
        print("Tests finished")
    }


}

