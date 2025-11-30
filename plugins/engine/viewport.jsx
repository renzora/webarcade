import { onMount, onCleanup } from 'solid-js';
import { Engine, Scene, ArcRotateCamera, Vector3, HemisphericLight, MeshBuilder, StandardMaterial, Color3, Color4 } from '@babylonjs/core';

export default function Viewport() {
    let canvasRef;

    onMount(() => {
        const engine = new Engine(canvasRef, true, {
            preserveDrawingBuffer: true,
            stencil: true
        });

        // Let Babylon auto-resize based on canvas CSS size
        engine.setSize(canvasRef.clientWidth, canvasRef.clientHeight);

        const scene = new Scene(engine);
        scene.clearColor = new Color4(0.1, 0.1, 0.18, 1);

        // Camera
        const camera = new ArcRotateCamera('camera', Math.PI / 4, Math.PI / 3, 6, Vector3.Zero(), scene);
        camera.attachControl(canvasRef, true);
        camera.lowerRadiusLimit = 2;
        camera.upperRadiusLimit = 20;

        // Light
        const light = new HemisphericLight('light', new Vector3(0, 1, 0), scene);
        light.intensity = 0.9;

        // Grid
        const ground = MeshBuilder.CreateGround('ground', { width: 10, height: 10, subdivisions: 10 }, scene);
        const groundMaterial = new StandardMaterial('groundMat', scene);
        groundMaterial.wireframe = true;
        groundMaterial.emissiveColor = new Color3(0.25, 0.25, 0.25);
        ground.material = groundMaterial;

        // Cube
        const cube = MeshBuilder.CreateBox('cube', { size: 1 }, scene);
        cube.position.y = 0.5;
        const cubeMaterial = new StandardMaterial('cubeMat', scene);
        cubeMaterial.diffuseColor = new Color3(0.29, 0.56, 0.85);
        cubeMaterial.specularColor = new Color3(0.4, 0.4, 0.4);
        cube.material = cubeMaterial;

        // Cube edges
        cube.enableEdgesRendering();
        cube.edgesWidth = 2.0;
        cube.edgesColor = new Color4(0.53, 0.8, 1, 1);

        // Render loop - check for size changes each frame
        let lastWidth = canvasRef.clientWidth;
        let lastHeight = canvasRef.clientHeight;

        engine.runRenderLoop(() => {
            const width = canvasRef.clientWidth;
            const height = canvasRef.clientHeight;

            if (width !== lastWidth || height !== lastHeight) {
                lastWidth = width;
                lastHeight = height;
                engine.setSize(width, height);
            }

            scene.render();
        });

        // Cleanup
        onCleanup(() => {
            engine.dispose();
        });
    });

    return (
        <canvas
            ref={canvasRef}
            style={{
                width: '100%',
                height: '100%',
                display: 'block',
                outline: 'none'
            }}
        />
    );
}
