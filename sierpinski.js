import * as THREE from 'https://unpkg.com/three/build/three.module.js'

export function tetrahedron(base, scale)
{
    // [-1/sqrt(3) , +-1, 0] * sqrt(3)/2
    // [2/sqrt(3),] * sqrt(3)/2
    // [0,0,4/sqrt(6)] * sqrt(3)/2 = [0,0,2/sqrt(2) ]
    
    let x,y,z
    [ x,y,z ] = base
    let dy = scale*Math.sqrt(3)/2
    let dx = scale* -0.5
    let p0 =  [ x+scale, y, z]
    let p1 = [x+dx, y+dy, z]
    let p2 = [x+dx, y-dy, z]
    let p3 = [x, y, z+scale*Math.sqrt(2)]
    return p0.concat(p1,p2,p3)
    /*return p0.concat(p2,p1,
	      p0,p1,p3,
	      p0,p3,p2,
	      p1,p2,p3
	     )*/
}

export function geometry()
{
    let vertices = new Float32Array(tetrahedron([0,0,0], 0.2))
    
    console.log(vertices)
    
    let geometry = new THREE.BufferGeometry()
    let indices = [0,2,1,
		   0,1,3,
		   0,3,2,
		   1,2,3,]
	/*let indices =[
		       0,1,2,
		       3,4,5,
		       6,7,8,
		       9,10,11
		       ]*/
    geometry.setAttribute('position', new THREE.BufferAttribute(vertices, 3))
    geometry.setIndex(indices)

    let vertex_colors = new Float32Array([1,0,0,
					  0,1,0,
					  0,0,1,
					  1,1,0,])
    // geometry.setAttribute('color', new THREE.BufferAttribute(vertex_colors, 3))
    
    geometry.computeVertexNormals()
    return geometry
}

let x
export default x
