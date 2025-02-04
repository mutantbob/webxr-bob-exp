import * as THREE from 'https://unpkg.com/three/build/three.module.js'

import {sierpinski} from "./rust-pkg/sierpinski.js"

export function geometry()
{
    let levels = 6;
    let vertices = new Float32Array(sierpinski([0,0,0], 0.2, levels))
    //console.log(vertices)
    
    let geometry = new THREE.BufferGeometry()

    geometry.setAttribute('position', new THREE.BufferAttribute(vertices, 3))

    let indices =[ ]
    for (let i =0; i< vertices.length/3 ; i++) {
	indices.push(i)
    }
    //console.log(indices)

    geometry.setIndex(
	new THREE.BufferAttribute(
	    new Uint16Array(indices),1
	)
    )

    /*let vertex_colors = new Float32Array([1,0,0,
      0,1,0,
      0,0,1,
      1,1,0,])
      geometry.setAttribute('color', new THREE.BufferAttribute(vertex_colors, 3))
    */
    
    geometry.computeVertexNormals()
    return geometry
}

let x
export default x
