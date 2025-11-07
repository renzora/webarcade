import { render } from 'solid-js/web';
import { createSignal, createEffect, onCleanup, Show } from 'solid-js';
import * as BABYLON from '@babylonjs/core';
import '@/index.css';
import { WEBARCADE_WS } from '@/api/bridge';

const DISPLAY_DURATION = 6000;

function AlertsOverlay() {
  const [isConnected, setIsConnected] = createSignal(false);
  const [alertQueue, setAlertQueue] = createSignal([]);
  const [currentAlert, setCurrentAlert] = createSignal(null);
  const [animationPhase, setAnimationPhase] = createSignal('hidden');

  let ws;
  let canvasRef;
  let engine;
  let scene;
  let camera;
  let glowLayer;
  let hideTimeout;
  let shapes = [];
  let particleSystems = [];
  let currentAnimationObserver = null;
  let cardBackgroundPlane;
  const [cardData, setCardData] = createSignal(null);
  const [cardVisible, setCardVisible] = createSignal(false);

  const alertConfigs = {
    follow: {
      title: 'NEW FOLLOWER',
      emoji: '❤️',
      gradient: 'from-pink-500 via-orange-500 to-pink-500',
      shapeType: 'hearts',
      color1: new BABYLON.Color3(1, 0.2, 0.5),
      color2: new BABYLON.Color3(1, 0.5, 0),
      particleColor1: new BABYLON.Color4(1, 0.2, 0.5, 1),
      particleColor2: new BABYLON.Color4(1, 0.5, 0, 1)
    },
    sub: {
      title: 'NEW SUBSCRIBER',
      emoji: '⭐',
      gradient: 'from-blue-500 via-cyan-500 to-purple-500',
      shapeType: 'stars',
      color1: new BABYLON.Color3(0, 0.8, 1),
      color2: new BABYLON.Color3(0.4, 0.2, 1),
      particleColor1: new BABYLON.Color4(0, 0.8, 1, 1),
      particleColor2: new BABYLON.Color4(0.4, 0.2, 1, 1)
    },
    resub: {
      title: 'RESUBSCRIBED',
      emoji: '🎉',
      gradient: 'from-purple-500 via-pink-500 to-orange-500',
      shapeType: 'stars',
      color1: new BABYLON.Color3(0.8, 0.3, 1),
      color2: new BABYLON.Color3(1, 0.5, 0),
      particleColor1: new BABYLON.Color4(0.8, 0.3, 1, 1),
      particleColor2: new BABYLON.Color4(1, 0.5, 0, 1)
    },
    gift_sub: {
      title: 'GIFT SUB',
      emoji: '🎁',
      gradient: 'from-yellow-500 via-orange-500 to-pink-500',
      shapeType: 'boxes',
      color1: new BABYLON.Color3(1, 0.8, 0),
      color2: new BABYLON.Color3(1, 0.3, 0.5),
      particleColor1: new BABYLON.Color4(1, 0.8, 0, 1),
      particleColor2: new BABYLON.Color4(1, 0.3, 0.5, 1)
    },
    raid: {
      title: 'RAID',
      emoji: '⚔️',
      gradient: 'from-green-500 via-cyan-500 to-blue-500',
      shapeType: 'cubes',
      color1: new BABYLON.Color3(0, 1, 0.3),
      color2: new BABYLON.Color3(0, 0.6, 1),
      particleColor1: new BABYLON.Color4(0, 1, 0.5, 1),
      particleColor2: new BABYLON.Color4(0, 0.6, 1, 1)
    },
    bits: {
      title: 'BITS',
      emoji: '💎',
      gradient: 'from-purple-500 via-pink-500 to-yellow-500',
      shapeType: 'diamonds',
      color1: new BABYLON.Color3(0.7, 0.3, 1),
      color2: new BABYLON.Color3(1, 0.8, 0),
      particleColor1: new BABYLON.Color4(0.7, 0.3, 1, 1),
      particleColor2: new BABYLON.Color4(1, 0.8, 0, 1)
    },
    channel_points: {
      title: 'CHANNEL POINTS',
      emoji: '✨',
      gradient: 'from-cyan-500 via-blue-500 to-purple-500',
      shapeType: 'orbs',
      color1: new BABYLON.Color3(0, 1, 1),
      color2: new BABYLON.Color3(0.4, 0.2, 1),
      particleColor1: new BABYLON.Color4(0, 1, 1, 1),
      particleColor2: new BABYLON.Color4(0.4, 0.2, 1, 1)
    }
  };

  const connectWebSocket = () => {
    ws = new WebSocket(WEBARCADE_WS);

    ws.onopen = () => {
      console.log('🟢 WebSocket connected to', WEBARCADE_WS);
      setIsConnected(true);
      ws.send(JSON.stringify({ type: 'subscribe', channels: ['twitch'] }));
      console.log('📤 Sent subscription request for twitch channel');
    };

    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        console.log('📨 WebSocket message received:', data);

        // Handle different message types
        if (data.type === 'connected') {
          console.log('✅ WebSocket connection confirmed');
          return;
        }

        // Handle alert.show events from alerts plugin
        if (data.event_type === 'alert.show' && data.payload) {
          console.log('🔔 ALERT.SHOW EVENT RECEIVED!');
          console.log('📦 Alert payload:', data.payload);
          console.log('🕒 Timestamp:', data.timestamp);
          console.log('🔌 Source plugin:', data.source_plugin);
          handleTwitchEvent(data.payload);
          return;
        }

        if (data.type === 'twitch_event' && data.event) {
          console.log('📡 Received twitch_event, extracting event:', data.event);
          handleTwitchEvent(data.event);
          return;
        }

        // Direct TwitchEvent format
        if (data.type) {
          console.log('🎯 Direct event type:', data.type);
          handleTwitchEvent(data);
        } else if (data.channel === 'twitch' && data.event) {
          console.log('📺 Channel event:', data.event);
          handleTwitchEvent(data.event);
        } else {
          console.log('⚠️ Unhandled WebSocket message format:', data);
        }
      } catch (error) {
        console.error('❌ WebSocket message error:', error);
      }
    };

    ws.onclose = () => {
      console.log('WebSocket disconnected');
      setIsConnected(false);
      setTimeout(connectWebSocket, 3000);
    };

    ws.onerror = (error) => {
      console.error('WebSocket error:', error);
    };
  };

  const handleTwitchEvent = (event) => {
    console.log('🎬 handleTwitchEvent called with event:', event);
    console.log('📋 Event type:', event.type);
    let alert;

    switch (event.type) {
      case 'follow':
        console.log('💙 Processing FOLLOW event');
        alert = {
          type: 'follow',
          username: event.user_name || event.display_name || event.username,
          message: 'Thanks for the follow!'
        };
        break;

      case 'subscription':
        console.log('⭐ Processing SUBSCRIPTION event');
        alert = {
          type: 'sub',
          username: event.user_name || event.display_name || event.username,
          tier: event.tier || '1000',
          message: 'Thanks for subscribing!'
        };
        break;

      case 'resubscription':
        console.log('🎉 Processing RESUBSCRIPTION event');
        alert = {
          type: 'resub',
          username: event.user_name || event.display_name || event.username,
          months: event.cumulative_months || event.months || 1,
          tier: event.tier || '1000',
          message: event.message || ''
        };
        break;

      case 'gift_subscription':
        console.log('🎁 Processing GIFT SUBSCRIPTION event');
        alert = {
          type: 'gift_sub',
          gifterName: event.user_name || event.gifter_display_name || event.gifter_name,
          recipientName: event.recipient_user_name || event.recipient_name || 'Someone',
          tier: event.tier || '1000',
          message: `Gifted to ${event.recipient_user_name || event.recipient_name || 'Someone'}!`
        };
        break;

      case 'raid':
        console.log('⚔️ Processing RAID event');
        alert = {
          type: 'raid',
          username: event.from_broadcaster_user_name || event.display_name,
          viewers: event.viewers || 1,
          message: `${event.viewers || 1} viewer${event.viewers !== 1 ? 's' : ''}!`
        };
        break;

      case 'cheer':
      case 'bits':
        console.log('💎 Processing BITS/CHEER event');
        alert = {
          type: 'bits',
          username: event.user_name || event.display_name || event.username,
          bits: event.bits || 1,
          message: `${event.bits || 1} bits!`
        };
        break;

      case 'channel_points_redemption':
      case 'channel_points':
        console.log('✨ Processing CHANNEL POINTS event');
        alert = {
          type: 'channel_points',
          username: event.user_name || event.display_name || event.username,
          reward: event.reward?.title || event.reward_title || 'Channel Points',
          message: event.reward?.title || event.reward_title || 'Redeemed!'
        };
        break;

      default:
        console.log('❓ Unknown event type:', event.type);
        console.log('Full event data:', event);
        return;
    }

    if (alert) {
      console.log('🎨 Alert created, calling handleAlert:', alert);
      handleAlert(alert);
    } else {
      console.log('⚠️ No alert object created!');
    }
  };

  const handleAlert = (alert) => {
    console.log('🔔🔔🔔 handleAlert called with:', alert);
    console.log('📊 Current queue before:', alertQueue());
    console.log('🎭 Current animation phase:', animationPhase());

    const newQueue = [...alertQueue(), alert];
    setAlertQueue(newQueue);
    console.log('📊 Queue updated to:', newQueue);
    console.log('📊 Queue length:', newQueue.length);

    if (animationPhase() === 'hidden') {
      console.log('✅ Animation phase is hidden, calling showNextAlert');
      showNextAlert();
    } else {
      console.log('⏳ Animation phase is NOT hidden (' + animationPhase() + '), alert queued');
    }
  };

  // Create shapes based on alert type
  const createShapes = (config) => {
    shapes.forEach(shape => shape.dispose());
    shapes = [];

    const count = 25;

    for (let i = 0; i < count; i++) {
      let shape;

      switch (config.shapeType) {
        case 'hearts':
          shape = BABYLON.MeshBuilder.CreateSphere(`shape_${i}`, { diameter: 1.2 }, scene);
          shape.scaling.y = 1.3;
          shape.scaling.x = 1.6; // Make wider
          shape.scaling.z = 1.6;
          break;
        case 'diamonds':
          shape = BABYLON.MeshBuilder.CreatePolyhedron(`shape_${i}`, { type: 2, size: 0.8 }, scene);
          shape.scaling.x = 1.5; // Make wider
          break;
        case 'stars':
          shape = BABYLON.MeshBuilder.CreatePolyhedron(`shape_${i}`, { type: 1, size: 1 }, scene);
          shape.scaling.x = 1.5; // Make wider
          break;
        case 'boxes':
          shape = BABYLON.MeshBuilder.CreateBox(`shape_${i}`, { size: 1.2 }, scene);
          shape.scaling.x = 1.8; // Make wider
          break;
        case 'cubes':
          shape = BABYLON.MeshBuilder.CreateBox(`shape_${i}`, { size: 1 }, scene);
          shape.scaling.x = 1.7; // Make wider
          break;
        case 'orbs':
        default:
          shape = BABYLON.MeshBuilder.CreateSphere(`shape_${i}`, { diameter: 1.2 }, scene);
          shape.scaling.x = 1.5; // Make wider
          break;
      }

      const mat = new BABYLON.PBRMaterial(`shapeMat_${i}`, scene);
      mat.albedoColor = config.color1;
      mat.emissiveColor = config.color1.scale(0.4);
      mat.metallic = 0.8;
      mat.roughness = 0.2;
      shape.material = mat;

      const angle = (i / count) * Math.PI * 2;
      const radius = 15 + Math.random() * 10; // Wider spread
      const yOffset = (Math.random() - 0.5) * 12; // More vertical spread

      shape.position = new BABYLON.Vector3(
        Math.cos(angle) * radius,
        yOffset,
        Math.sin(angle) * radius
      );

      shape.userData = {
        angle: angle,
        radius: radius,
        yOffset: yOffset,
        orbitSpeed: 0.01 + Math.random() * 0.005,
        bobSpeed: 0.6 + Math.random() * 0.4,
        bobOffset: Math.random() * Math.PI * 2,
        verticalSpeed: 0.3 + Math.random() * 0.3,
        verticalOffset: Math.random() * Math.PI * 2,
        pulseSpeed: 1 + Math.random() * 0.5,
        pulseOffset: Math.random() * Math.PI * 2
      };

      shapes.push(shape);
    }
  };

  const createParticles = (config) => {
    particleSystems.forEach(ps => ps.dispose());
    particleSystems = [];

    // Main particle system
    const particles = new BABYLON.ParticleSystem('particles', 300, scene);
    particles.particleTexture = new BABYLON.Texture('https://www.babylonjs.com/assets/Flare.png', scene);
    particles.emitter = new BABYLON.Vector3(0, 0, 0);
    particles.minEmitBox = new BABYLON.Vector3(-8, -6, -8);
    particles.maxEmitBox = new BABYLON.Vector3(8, 6, 8);
    particles.color1 = config.particleColor1;
    particles.color2 = config.particleColor2;
    particles.colorDead = new BABYLON.Color4(0, 0, 0, 0);
    particles.minSize = 0.2;
    particles.maxSize = 0.6;
    particles.minLifeTime = 1;
    particles.maxLifeTime = 2;
    particles.emitRate = 50;
    particles.blendMode = BABYLON.ParticleSystem.BLENDMODE_ONEONE;
    particles.gravity = new BABYLON.Vector3(0, 1, 0);
    particles.direction1 = new BABYLON.Vector3(-2, 1, -2);
    particles.direction2 = new BABYLON.Vector3(2, 3, 2);
    particles.minEmitPower = 1;
    particles.maxEmitPower = 2;
    particles.updateSpeed = 0.02;
    particles.start();
    particleSystems.push(particles);

    // Sparkle particles
    const sparkles = new BABYLON.ParticleSystem('sparkles', 200, scene);
    sparkles.particleTexture = new BABYLON.Texture('https://www.babylonjs.com/assets/Flare.png', scene);
    sparkles.emitter = new BABYLON.Vector3(0, 0, 0);
    sparkles.minEmitBox = new BABYLON.Vector3(-12, -8, -12);
    sparkles.maxEmitBox = new BABYLON.Vector3(12, 8, 12);
    sparkles.color1 = new BABYLON.Color4(1, 1, 1, 1);
    sparkles.color2 = config.particleColor1;
    sparkles.colorDead = new BABYLON.Color4(0, 0, 0, 0);
    sparkles.minSize = 0.1;
    sparkles.maxSize = 0.3;
    sparkles.minLifeTime = 0.5;
    sparkles.maxLifeTime = 1.5;
    sparkles.emitRate = 80;
    sparkles.blendMode = BABYLON.ParticleSystem.BLENDMODE_ONEONE;
    sparkles.gravity = new BABYLON.Vector3(0, 0.5, 0);
    sparkles.direction1 = new BABYLON.Vector3(-1, 0.5, -1);
    sparkles.direction2 = new BABYLON.Vector3(1, 2, 1);
    sparkles.minEmitPower = 0.5;
    sparkles.maxEmitPower = 1.5;
    sparkles.updateSpeed = 0.01;
    sparkles.start();
    particleSystems.push(sparkles);

    // Trail particles
    const trails = new BABYLON.ParticleSystem('trails', 150, scene);
    trails.particleTexture = new BABYLON.Texture('https://www.babylonjs.com/assets/Flare.png', scene);
    trails.emitter = new BABYLON.Vector3(0, 0, 0);
    trails.createDirectedSphereEmitter(15, new BABYLON.Vector3(0, 1, 0), new BABYLON.Vector3(0, 1, 0));
    trails.color1 = config.particleColor2;
    trails.color2 = config.particleColor1;
    trails.colorDead = new BABYLON.Color4(0, 0, 0, 0);
    trails.minSize = 0.15;
    trails.maxSize = 0.4;
    trails.minLifeTime = 2;
    trails.maxLifeTime = 3;
    trails.emitRate = 30;
    trails.blendMode = BABYLON.ParticleSystem.BLENDMODE_ONEONE;
    trails.minEmitPower = 2;
    trails.maxEmitPower = 4;
    trails.updateSpeed = 0.02;
    trails.start();
    particleSystems.push(trails);
  };

  const animateScene = () => {
    if (currentAnimationObserver) {
      scene.onBeforeRenderObservable.remove(currentAnimationObserver);
    }

    let startTime = Date.now();
    let exitStartTime = null;
    const exitDuration = 600;

    currentAnimationObserver = scene.onBeforeRenderObservable.add(() => {
      const elapsed = (Date.now() - startTime) / 1000;

      if (animationPhase() === 'hidden') {
        scene.onBeforeRenderObservable.remove(currentAnimationObserver);
        return;
      }

      // Handle fade-out during exiting phase
      if (animationPhase() === 'exiting') {
        if (!exitStartTime) {
          exitStartTime = Date.now();
        }

        const exitElapsed = Date.now() - exitStartTime;
        const fadeProgress = Math.min(exitElapsed / exitDuration, 1);
        const alpha = 1 - fadeProgress;

        shapes.forEach((shape) => {
          if (shape.material) {
            shape.material.alpha = alpha;
            shape.material.transparencyMode = BABYLON.Material.MATERIAL_ALPHABLEND;
          }
        });

        // Fade out particles
        particleSystems.forEach((ps) => {
          ps.emitRate = ps.emitRate * 0.95;
        });
      }

      // Gentle camera rotation
      if (camera) {
        const cameraAngle = elapsed * 0.05;
        camera.position.x = Math.cos(cameraAngle) * 45;
        camera.position.z = Math.sin(cameraAngle) * 45;
        camera.position.y = 8;
        camera.setTarget(BABYLON.Vector3.Zero());
      }

      // Enhanced shape animation
      shapes.forEach((shape) => {
        // Orbital motion
        shape.userData.angle += shape.userData.orbitSpeed;
        shape.position.x = Math.cos(shape.userData.angle) * shape.userData.radius;
        shape.position.z = Math.sin(shape.userData.angle) * shape.userData.radius;

        // Vertical movement - smooth up and down motion
        const verticalMotion = Math.sin(elapsed * shape.userData.verticalSpeed + shape.userData.verticalOffset) * 3;
        const bobMotion = Math.sin(elapsed * shape.userData.bobSpeed + shape.userData.bobOffset) * 0.8;
        shape.position.y = shape.userData.yOffset + verticalMotion + bobMotion;

        // Rotation
        shape.rotation.y += 0.008;
        shape.rotation.x += 0.005;

        // Pulsing scale effect
        const basePulse = 1 + Math.sin(elapsed * shape.userData.pulseSpeed + shape.userData.pulseOffset) * 0.15;
        const currentScaleX = shape.scaling.x / (shape.userData.baseScaleX || shape.scaling.x);
        const currentScaleY = shape.scaling.y / (shape.userData.baseScaleY || shape.scaling.y);
        const currentScaleZ = shape.scaling.z / (shape.userData.baseScaleZ || shape.scaling.z);

        if (!shape.userData.baseScaleX) {
          shape.userData.baseScaleX = shape.scaling.x;
          shape.userData.baseScaleY = shape.scaling.y;
          shape.userData.baseScaleZ = shape.scaling.z;
        }

        shape.scaling.x = shape.userData.baseScaleX * basePulse;
        shape.scaling.y = shape.userData.baseScaleY * basePulse;
        shape.scaling.z = shape.userData.baseScaleZ * basePulse;

        // Subtle emissive intensity variation
        if (shape.material && shape.material.emissiveColor) {
          const emissiveBoost = 1 + Math.sin(elapsed * 2 + shape.userData.pulseOffset) * 0.2;
          const baseEmissive = shape.userData.baseEmissive || shape.material.emissiveColor.clone();
          if (!shape.userData.baseEmissive) {
            shape.userData.baseEmissive = baseEmissive;
          }
          shape.material.emissiveColor = baseEmissive.scale(emissiveBoost);
        }
      });
    });
  };

  const showNextAlert = () => {
    console.log('🎬🎬🎬 showNextAlert called');
    const queue = alertQueue();
    console.log('📊 Current queue:', queue);
    console.log('📊 Queue length:', queue.length);

    if (queue.length === 0) {
      console.log('⚠️ Queue is empty, returning');
      return;
    }

    const [next, ...rest] = queue;
    console.log('🎯 Showing alert:', next);
    console.log('📊 Remaining queue:', rest);

    setAlertQueue(rest);
    setCurrentAlert(next);
    console.log('✅ Current alert set to:', next);

    const config = alertConfigs[next.type] || alertConfigs.follow;

    // Create 3D scene
    if (scene) {
      createShapes(config);
      createParticles(config);
      animateScene();
    }

    // Update and show card
    updateCard(next, config);
    showCard();

    setAnimationPhase('showing');

    if (hideTimeout) clearTimeout(hideTimeout);

    hideTimeout = setTimeout(() => {
      setAnimationPhase('exiting');
      hideCard();

      setTimeout(() => {
        shapes.forEach(shape => shape.dispose());
        shapes = [];
        particleSystems.forEach(ps => {
          ps.stop();
          ps.dispose();
        });
        particleSystems = [];

        setAnimationPhase('hidden');
        setCurrentAlert(null);
        setTimeout(() => showNextAlert(), 300);
      }, 600);
    }, DISPLAY_DURATION);
  };

  const initBabylon = () => {
    if (!canvasRef) return;

    canvasRef.width = window.innerWidth;
    canvasRef.height = window.innerHeight;

    engine = new BABYLON.Engine(canvasRef, true);
    scene = new BABYLON.Scene(engine);
    scene.clearColor = new BABYLON.Color4(0, 0, 0, 0);

    camera = new BABYLON.ArcRotateCamera('camera', 0, Math.PI / 2, 45, BABYLON.Vector3.Zero(), scene);
    camera.lowerRadiusLimit = 35;
    camera.upperRadiusLimit = 55;

    const light1 = new BABYLON.HemisphericLight('light1', new BABYLON.Vector3(0, 1, 0), scene);
    light1.intensity = 0.7;

    const light2 = new BABYLON.PointLight('light2', new BABYLON.Vector3(0, 15, 15), scene);
    light2.intensity = 1.5;

    glowLayer = new BABYLON.GlowLayer('glow', scene);
    glowLayer.intensity = 0.5;
    glowLayer.blurKernelSize = 48;

    engine.runRenderLoop(() => scene.render());

    window.addEventListener('resize', () => {
      canvasRef.width = window.innerWidth;
      canvasRef.height = window.innerHeight;
      engine.resize();
    });

    // Create 3D background plane for card (positioned behind shapes)
    cardBackgroundPlane = BABYLON.MeshBuilder.CreatePlane('cardBackground', { width: 18, height: 12 }, scene);
    cardBackgroundPlane.position.z = -8;
    cardBackgroundPlane.position.y = 3;

    const cardBgMat = new BABYLON.StandardMaterial('cardBgMat', scene);
    cardBgMat.diffuseColor = new BABYLON.Color3(0.2, 0.2, 0.3);
    cardBgMat.emissiveColor = new BABYLON.Color3(0.1, 0.1, 0.15);
    cardBgMat.alpha = 0;
    cardBgMat.transparencyMode = BABYLON.Material.MATERIAL_ALPHABLEND;
    cardBackgroundPlane.material = cardBgMat;
  };

  const updateCard = (alert, config) => {
    const username = alert.username || alert.gifterName || 'User';

    let detail = '';
    if (alert.type === 'bits' && alert.bits) {
      detail = `${alert.bits} bits`;
    } else if (alert.type === 'raid' && alert.viewers) {
      detail = `${alert.viewers} viewer${alert.viewers !== 1 ? 's' : ''}`;
    } else if (alert.type === 'resub' && alert.months) {
      detail = `${alert.months} month${alert.months !== 1 ? 's' : ''}`;
    } else if (alert.type === 'gift_sub' && alert.recipientName) {
      detail = `For ${alert.recipientName}`;
    }

    const color1 = `rgb(${config.color1.r * 255}, ${config.color1.g * 255}, ${config.color1.b * 255})`;
    const color2 = `rgb(${config.color2.r * 255}, ${config.color2.g * 255}, ${config.color2.b * 255})`;

    setCardData({
      emoji: config.emoji,
      title: config.title,
      username: username.toUpperCase(),
      detail: detail,
      color1: color1,
      color2: color2
    });

    // Update 3D background plane color
    if (cardBackgroundPlane && cardBackgroundPlane.material) {
      cardBackgroundPlane.material.emissiveColor = config.color1.scale(0.3);
    }
  };

  const showCard = () => {
    setCardVisible(true);
  };

  const hideCard = () => {
    setCardVisible(false);
  };

  // Track signal changes
  createEffect(() => {
    console.log('[SIGNAL] currentAlert changed to:', currentAlert());
  });

  createEffect(() => {
    console.log('[SIGNAL] animationPhase changed to:', animationPhase());
  });

  createEffect(() => {
    console.log('═══════════════════════════════════════════');
    console.log('🚀 ALERTS OVERLAY INITIALIZED');
    console.log('═══════════════════════════════════════════');
    console.log('🎨 Initializing Babylon.js 3D engine...');
    initBabylon();
    console.log('🔌 Connecting to WebSocket:', WEBARCADE_WS);
    connectWebSocket();

    // Expose test function
    window.testAlert = (type = 'follow') => {
      console.log('testAlert called with type:', type);
      const testAlerts = {
        follow: { type: 'follow', username: 'TestUser', message: 'Test!' },
        sub: { type: 'sub', username: 'TestSub', tier: '1000', message: 'Test!' },
        bits: { type: 'bits', username: 'TestBits', bits: 100, message: '100 bits!' },
        raid: { type: 'raid', username: 'TestRaider', viewers: 50, message: '50 viewers!' },
        resub: { type: 'resub', username: 'TestResub', months: 12, message: '12 months!' },
        gift_sub: { type: 'gift_sub', gifterName: 'TestGifter', recipientName: 'Lucky', message: 'Gift!' },
        channel_points: { type: 'channel_points', username: 'TestPoints', reward: 'Hydrate!', message: 'Hydrate!' }
      };
      handleAlert(testAlerts[type] || testAlerts.follow);
    };
    console.log('window.testAlert function exposed');

    onCleanup(() => {
      if (ws) ws.close();
      if (engine) engine.dispose();
      if (hideTimeout) clearTimeout(hideTimeout);
    });
  });

  return (
    <div class="fixed inset-0 pointer-events-none bg-transparent">
      {/* Babylon.js 3D Canvas */}
      <canvas ref={canvasRef} class="w-full h-full absolute inset-0 z-50" style={{ display: 'block' }} />

      {/* HTML Alert Card */}
      <Show when={cardData()}>
        <style>
          {`
            @keyframes shine {
              0% { left: -150%; }
              50% { left: 150%; }
              100% { left: 150%; }
            }
            @keyframes transformerEnter {
              0% {
                opacity: 0;
                transform: scale(0.3) rotateX(90deg) rotateZ(45deg);
                filter: brightness(3);
              }
              40% {
                opacity: 1;
                transform: scale(1.15) rotateX(-10deg) rotateZ(-5deg);
                filter: brightness(1.5);
              }
              60% {
                transform: scale(0.95) rotateX(5deg) rotateZ(2deg);
                filter: brightness(1.2);
              }
              80% {
                transform: scale(1.05) rotateX(-2deg) rotateZ(-1deg);
                filter: brightness(1.1);
              }
              100% {
                opacity: 1;
                transform: scale(1) rotateX(0deg) rotateZ(0deg);
                filter: brightness(1);
              }
            }
            @keyframes slidePanel {
              0% {
                clip-path: inset(0 100% 0 0);
              }
              100% {
                clip-path: inset(0 0 0 0);
              }
            }
            @keyframes glitchPulse {
              0%, 100% {
                text-shadow: 2px 2px 4px rgba(0,0,0,0.7);
              }
              50% {
                text-shadow:
                  2px 2px 4px rgba(0,0,0,0.7),
                  0 0 10px rgba(255,255,255,0.5),
                  0 0 20px rgba(255,255,255,0.3);
              }
            }
            .card-shine {
              position: relative;
              overflow: hidden;
              perspective: 1000px;
              transform-style: preserve-3d;
            }
            .card-shine::before {
              content: '';
              position: absolute;
              top: -50%;
              left: -150%;
              width: 100%;
              height: 200%;
              background: linear-gradient(90deg, transparent, rgba(255,255,255,0.3), transparent);
              transform: skewX(-25deg);
              animation: shine 3s ease-in-out infinite;
              pointer-events: none;
            }
            .card-shine::after {
              content: '';
              position: absolute;
              inset: 0;
              background: linear-gradient(90deg,
                transparent 0%,
                rgba(255,255,255,0.1) 25%,
                rgba(255,255,255,0.2) 50%,
                rgba(255,255,255,0.1) 75%,
                transparent 100%);
              animation: slidePanel 0.6s ease-out;
              pointer-events: none;
            }
            .transformer-enter {
              animation: transformerEnter 0.8s cubic-bezier(0.68, -0.55, 0.265, 1.55);
            }
            .content-glitch {
              animation: glitchPulse 0.8s ease-out;
            }
          `}
        </style>
        <div
          class={`card-shine absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[500px] h-[300px] rounded-3xl shadow-xl z-[55] flex flex-col items-center justify-center gap-4 ${
            cardVisible() ? 'transformer-enter' : 'opacity-0'
          }`}
          style={{
            background: `linear-gradient(135deg, ${cardData().color1}, ${cardData().color2})`,
            'backdrop-filter': 'blur(5px)',
            'box-shadow': '0 0 40px rgba(0,0,0,0.6), 0 0 80px rgba(255,255,255,0.2)'
          }}
        >
          <div class={`text-7xl ${cardVisible() ? 'content-glitch' : ''}`}>{cardData().emoji}</div>
          <div class={`text-white text-2xl font-bold ${cardVisible() ? 'content-glitch' : ''}`} style="text-shadow: 2px 2px 4px rgba(0,0,0,0.7)">
            {cardData().title}
          </div>
          <div class={`text-white text-5xl font-bold ${cardVisible() ? 'content-glitch' : ''}`} style="text-shadow: 3px 3px 6px rgba(0,0,0,0.7)">
            {cardData().username}
          </div>
          <div class={`text-white/90 text-xl font-semibold ${cardVisible() ? 'content-glitch' : ''}`} style="text-shadow: 2px 2px 4px rgba(0,0,0,0.7)">
            {cardData().detail}
          </div>
        </div>
      </Show>

      {/* Debug Info */}
      <Show when={currentAlert()}>
        <div class="absolute bottom-4 left-4 text-white text-sm font-mono bg-black/60 backdrop-blur-md px-4 py-3 rounded-xl border border-white/20 shadow-lg z-[60]">
          <div class="font-bold mb-1 text-cyan-300">Alerts</div>
          <div>Phase: <span class="text-yellow-300">{animationPhase()}</span></div>
          <div>Type: <span class="text-green-300">{currentAlert()?.type}</span></div>
          <div>Queue: <span class="text-purple-300">{alertQueue().length}</span></div>
        </div>
      </Show>
    </div>
  );
}

const root = document.getElementById('root');
if (root) {
  render(() => <AlertsOverlay />, root);
}
