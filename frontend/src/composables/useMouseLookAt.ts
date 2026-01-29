import { ref, onMounted, onUnmounted, type Ref } from 'vue';

interface MouseLookAtOptions {
  proximityRadius?: number;
  maxRotationX?: number;
  maxRotationY?: number;
  smoothing?: number;
}

interface MouseLookAtReturn {
  rotationX: Ref<number>;
  rotationY: Ref<number>;
  isNearby: Ref<boolean>;
  containerRef: Ref<HTMLElement | null>;
}

export function useMouseLookAt(
  options: MouseLookAtOptions = {}
): MouseLookAtReturn {
  const {
    proximityRadius = 150,
    maxRotationX = 0.9,
    maxRotationY = 0.9,
    smoothing = 0.1,
  } = options;

  const containerRef = ref<HTMLElement | null>(null);
  const rotationX = ref(0);
  const rotationY = ref(0);
  const isNearby = ref(false);

  let targetRotationX = 0;
  let targetRotationY = 0;
  let animationFrameId: number | null = null;
  let lastUpdate = 0;
  const throttleMs = 100; // 10fps

  // Cache bounding rect to avoid layout thrashing
  let cachedCenter: { x: number; y: number } | null = null;

  function updateCachedRect() {
    if (!containerRef.value) {
      cachedCenter = null;
      return;
    }
    const rect = containerRef.value.getBoundingClientRect();
    cachedCenter = {
      x: rect.left + rect.width / 2,
      y: rect.top + rect.height / 2,
    };
  }

  function getElementCenter(): { x: number; y: number } | null {
    return cachedCenter;
  }

  function handleMouseMove(event: MouseEvent) {
    const now = performance.now();
    if (now - lastUpdate < throttleMs) return;
    lastUpdate = now;

    const center = getElementCenter();
    if (!center) return;

    const dx = event.clientX - center.x;
    const dy = event.clientY - center.y;
    const distance = Math.sqrt(dx * dx + dy * dy);

    if (distance <= proximityRadius) {
      isNearby.value = true;
      const normalizedX = (dx * 1.5) / proximityRadius;
      const normalizedY = (dy * 1.5) / proximityRadius;
      targetRotationY = normalizedX * maxRotationY;
      targetRotationX = normalizedY * maxRotationX;
    } else {
      isNearby.value = false;
      targetRotationX = 0;
      targetRotationY = 0;
    }
  }

  function animate() {
    rotationX.value += (targetRotationX - rotationX.value) * smoothing;
    rotationY.value += (targetRotationY - rotationY.value) * smoothing;

    if (
      Math.abs(targetRotationX - rotationX.value) > 0.001 ||
      Math.abs(targetRotationY - rotationY.value) > 0.001
    ) {
      animationFrameId = requestAnimationFrame(animate);
    } else {
      rotationX.value = targetRotationX;
      rotationY.value = targetRotationY;
      animationFrameId = null;
    }
  }

  function startAnimation() {
    if (animationFrameId === null) {
      animationFrameId = requestAnimationFrame(animate);
    }
  }

  function onMouseMoveWrapper(event: MouseEvent) {
    handleMouseMove(event);
    startAnimation();
  }

  onMounted(() => {
    updateCachedRect();
    window.addEventListener('mousemove', onMouseMoveWrapper, { passive: true });
    window.addEventListener('scroll', updateCachedRect, { passive: true });
    window.addEventListener('resize', updateCachedRect, { passive: true });
  });

  onUnmounted(() => {
    window.removeEventListener('mousemove', onMouseMoveWrapper);
    window.removeEventListener('scroll', updateCachedRect);
    window.removeEventListener('resize', updateCachedRect);
    if (animationFrameId !== null) {
      cancelAnimationFrame(animationFrameId);
    }
  });

  return {
    rotationX,
    rotationY,
    isNearby,
    containerRef,
  };
}
