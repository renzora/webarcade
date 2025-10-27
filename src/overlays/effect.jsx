import { render } from 'solid-js/web';
import { createSignal, createEffect, onCleanup, Show } from 'solid-js';
import * as BABYLON from '@babylonjs/core';
import '@/index.css';
import { WEBARCADE_WS } from '@/api/bridge';

function EffectOverlay() {
  const [isConnected, setIsConnected] = createSignal(false);
  const [isActive, setIsActive] = createSignal(false);

  let ws;
  let canvasRef;
  let engine;
  let scene;

  // Connect to WebSocket
  const connectWebSocket = () => {
    ws = new WebSocket(WEBARCADE_WS);

    ws.onopen = () => {
      console.log('✅ Connected to WebArcade (Effect Overlay)');
      setIsConnected(true);
    };

    ws.onclose = () => {
      console.log('❌ Disconnected');
      setIsConnected(false);
      setTimeout(connectWebSocket, 3000);
    };

    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        console.log('💥 Effect overlay received event:', data.type, data);

        // Handle Twitch events - check for effect trigger
        if (data.type === 'twitch_event' && data.event?.type === 'effect_trigger') {
          console.log('💥 Triggering crazy effect!');
          triggerEffect();
        }
      } catch (error) {
        console.error('Error parsing event:', error);
      }
    };
  };

  // Initialize Babylon.js scene
  const initBabylon = () => {
    if (!canvasRef) return;

    // Set canvas to full resolution
    canvasRef.width = window.innerWidth;
    canvasRef.height = window.innerHeight;

    engine = new BABYLON.Engine(canvasRef, true, {
      preserveDrawingBuffer: true,
      stencil: true,
      adaptToDeviceRatio: true  // Enable high DPI support
    });

    scene = new BABYLON.Scene(engine);
    scene.clearColor = new BABYLON.Color4(0, 0, 0, 0);

    // Create camera
    const camera = new BABYLON.ArcRotateCamera('camera', 0, 0, 0, BABYLON.Vector3.Zero(), scene);
    camera.setPosition(new BABYLON.Vector3(0, 5, -20));
    camera.attachControl(canvasRef, true);

    // Create lights
    const hemiLight = new BABYLON.HemisphericLight('hemiLight', new BABYLON.Vector3(0, 1, 0), scene);
    hemiLight.intensity = 0.7;

    const pointLight = new BABYLON.PointLight('pointLight', new BABYLON.Vector3(0, 10, 0), scene);
    pointLight.intensity = 0.5;

    // Run render loop
    engine.runRenderLoop(() => {
      scene.render();
    });

    // Handle window resize
    const handleResize = () => {
      canvasRef.width = window.innerWidth;
      canvasRef.height = window.innerHeight;
      engine.resize();
    };

    window.addEventListener('resize', handleResize);
  };

  // Create crazy 3D effect
  const triggerEffect = () => {
    if (!scene) return;

    setIsActive(true);

    // Clear previous objects
    scene.meshes.forEach(mesh => {
      if (mesh.name !== 'camera') {
        mesh.dispose();
      }
    });

    // Create multiple spinning objects
    const shapes = [];
    const colors = [
      new BABYLON.Color3(1, 0, 1),     // Magenta
      new BABYLON.Color3(0, 1, 1),     // Cyan
      new BABYLON.Color3(1, 1, 0),     // Yellow
      new BABYLON.Color3(1, 0.5, 0),   // Orange
      new BABYLON.Color3(0, 1, 0),     // Green
      new BABYLON.Color3(1, 0, 0),     // Red
    ];

    // Create torus knots
    for (let i = 0; i < 6; i++) {
      const torusKnot = BABYLON.MeshBuilder.CreateTorusKnot(`knot${i}`, {
        radius: 2,
        tube: 0.5,
        radialSegments: 128,
        tubularSegments: 64,
        p: 2,
        q: 3
      }, scene);

      const mat = new BABYLON.StandardMaterial(`mat${i}`, scene);
      mat.diffuseColor = colors[i];
      mat.specularColor = new BABYLON.Color3(1, 1, 1);
      mat.emissiveColor = colors[i].scale(0.5);
      torusKnot.material = mat;

      const angle = (i / 6) * Math.PI * 2;
      torusKnot.position = new BABYLON.Vector3(
        Math.cos(angle) * 8,
        Math.sin(angle * 2) * 3,
        Math.sin(angle) * 8
      );

      shapes.push(torusKnot);
    }

    // Create particle system
    const particleSystem = new BABYLON.ParticleSystem('particles', 2000, scene);
    particleSystem.particleTexture = new BABYLON.Texture('https://www.babylonjs.com/assets/Flare.png', scene);

    particleSystem.emitter = BABYLON.Vector3.Zero();
    particleSystem.minEmitBox = new BABYLON.Vector3(-10, -10, -10);
    particleSystem.maxEmitBox = new BABYLON.Vector3(10, 10, 10);

    particleSystem.color1 = new BABYLON.Color4(1, 0, 1, 1);
    particleSystem.color2 = new BABYLON.Color4(0, 1, 1, 1);
    particleSystem.colorDead = new BABYLON.Color4(0, 0, 0, 0);

    particleSystem.minSize = 0.1;
    particleSystem.maxSize = 1;

    particleSystem.minLifeTime = 1;
    particleSystem.maxLifeTime = 3;

    particleSystem.emitRate = 1000;

    particleSystem.blendMode = BABYLON.ParticleSystem.BLENDMODE_ONEONE;

    particleSystem.gravity = new BABYLON.Vector3(0, -9.81, 0);

    particleSystem.direction1 = new BABYLON.Vector3(-7, 8, 3);
    particleSystem.direction2 = new BABYLON.Vector3(7, 8, -3);

    particleSystem.minAngularSpeed = 0;
    particleSystem.maxAngularSpeed = Math.PI;

    particleSystem.minEmitPower = 1;
    particleSystem.maxEmitPower = 3;
    particleSystem.updateSpeed = 0.01;

    particleSystem.start();

    // Animate everything
    let time = 0;
    const cameraAnimation = scene.onBeforeRenderObservable.add(() => {
      time += 0.01;

      // Rotate shapes
      shapes.forEach((shape, i) => {
        shape.rotation.x += 0.02 * (i + 1);
        shape.rotation.y += 0.03 * (i + 1);
        shape.rotation.z += 0.01 * (i + 1);

        // Pulsate
        const scale = 1 + Math.sin(time * 2 + i) * 0.3;
        shape.scaling = new BABYLON.Vector3(scale, scale, scale);
      });

      // Crazy camera movement
      const camera = scene.activeCamera;
      camera.alpha = time * 0.5;
      camera.beta = Math.PI / 4 + Math.sin(time) * 0.3;
      camera.radius = 20 + Math.sin(time * 0.7) * 5;
    });

    // Auto-stop after 5 seconds
    setTimeout(() => {
      setIsActive(false);
      particleSystem.stop();
      scene.onBeforeRenderObservable.remove(cameraAnimation);

      setTimeout(() => {
        shapes.forEach(shape => shape.dispose());
        particleSystem.dispose();
      }, 2000);
    }, 5000);
  };

  createEffect(() => {
    connectWebSocket();
    initBabylon();

    onCleanup(() => {
      ws?.close();
      if (engine) {
        engine.dispose();
      }
    });
  });

  return (
    <div class="fixed inset-0 pointer-events-none overflow-hidden">
      <canvas
        ref={canvasRef}
        style={{
          display: isActive() ? 'block' : 'none',
          width: '100%',
          height: '100%',
          position: 'absolute',
          top: 0,
          left: 0
        }}
      />
    </div>
  );
}

// Only render when used as standalone (for OBS browser sources)
if (document.getElementById('root')) {
  render(() => <EffectOverlay />, document.getElementById('root'));
}
