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
    //return p0.concat(p1,p2,p3)
    return p0.concat(p2,p1,
	      p0,p1,p3,
	      p0,p3,p2,
	      p1,p2,p3
	     )
}

function average(p1, p2)
{
    return [
	(p1[0]+p2[0])*0.5,
	(p1[1]+p2[1])*0.5,
	(p1[2]+p2[2])*0.5,
     ]
}

export function sierpinski(base, scale, levels)
{
    if (levels>0) {
	let corners = tetrahedron(base,scale)
	let t1 = sierpinski(average(base, corners.slice(0,3)), scale*0.5, levels-1)
	let t2 = sierpinski(average(base, corners.slice(3,6)), scale*0.5, levels-1)
	let t3 = sierpinski(average(base, corners.slice(6,9)), scale*0.5, levels-1)
	let t4 = sierpinski(average(base, corners.slice(21,24)), scale*0.5, levels-1)

	return t1.concat(t2, t3, t4)
    } else {
	return tetrahedron(base, scale)
    }
}

export function geometry()
{
    let vertices = new Float32Array(sierpinski([0,0,0], 0.2, 2))
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
