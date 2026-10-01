import { useEffect, useRef } from "react";
import * as THREE from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
import { useReducedMotion } from "../utils/use-reduced-motion";
import { useIsMobile } from "../utils/is-mobile";

export interface SceneItem {
  title: string;
  color: string;
  onClick?: () => void;
}

/**
 * Raw Three.js canvas (no R3F reconciler) — reliable across React 18 builds.
 * Renders floating product "cards" as 3D boxes with labels.
 */
export const DesignCanvas = ({
  items = [],
  cameraPosition = [0, 0, 6],
}: {
  items?: SceneItem[];
  cameraPosition?: [number, number, number];
}) => {
  const mountRef = useRef<HTMLDivElement>(null);
  const reducedMotion = useReducedMotion();
  const isMobile = useIsMobile();

  useEffect(() => {
    const mount = mountRef.current;
    if (!mount) return;

    const width = mount.clientWidth || window.innerWidth;
    const height = mount.clientHeight || window.innerHeight;

    const scene = new THREE.Scene();
    scene.background = new THREE.Color(0xf3f4f6);

    const camera = new THREE.PerspectiveCamera(50, width / height, 0.1, 100);
    camera.position.set(cameraPosition[0], cameraPosition[1], cameraPosition[2]);

    const renderer = new THREE.WebGLRenderer({ antialias: true, alpha: false });
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    renderer.setSize(width, height);
    mount.appendChild(renderer.domElement);

    const controls = new OrbitControls(camera, renderer.domElement);
    controls.enableDamping = true;
    controls.dampingFactor = 0.08;
    controls.enablePan = false;
    controls.enableZoom = !isMobile;
    controls.rotateSpeed = isMobile ? 0.4 : 0.6;

    const ambient = new THREE.AmbientLight(0xffffff, 0.7);
    scene.add(ambient);
    const dir = new THREE.DirectionalLight(0xffffff, 0.8);
    dir.position.set(5, 5, 5);
    scene.add(dir);
    const dir2 = new THREE.DirectionalLight(0xffffff, 0.4);
    dir2.position.set(-5, -3, -5);
    scene.add(dir2);

    const raycaster = new THREE.Raycaster();
    const pointer = new THREE.Vector2();
    const cardMeshes: THREE.Mesh[] = [];

    // Layout cards in a grid
    const cols = items.length > 3 ? 3 : items.length || 1;
    const spacingX = 2.6;
    const spacingY = 2.0;
    const startX = -((cols - 1) * spacingX) / 2;

    items.forEach((item, i) => {
      const col = i % cols;
      const row = Math.floor(i / cols);
      const x = startX + col * spacingX;
      const y = (row * spacingY) / 1 - (Math.floor((items.length - 1) / cols) * spacingY) / 2;

      const group = new THREE.Group();
      group.position.set(x, -y, 0);

      const geo = new THREE.BoxGeometry(2, 1.2, 0.3);
      const mat = new THREE.MeshStandardMaterial({
        color: new THREE.Color(item.color),
        roughness: 0.45,
        metalness: 0.1,
      });
      const mesh = new THREE.Mesh(geo, mat);
      mesh.userData.onClick = item.onClick;
      group.add(mesh);
      cardMeshes.push(mesh);

      // Title label via canvas texture
      const label = makeLabel(item.title);
      const labelMat = new THREE.MeshBasicMaterial({ map: label, transparent: true });
      const labelMesh = new THREE.Mesh(new THREE.PlaneGeometry(1.9, 0.45), labelMat);
      labelMesh.position.set(0, 0, 0.16);
      group.add(labelMesh);

      scene.add(group);
    });

    function makeLabel(text: string): THREE.CanvasTexture {
      const c = document.createElement("canvas");
      c.width = 512;
      c.height = 128;
      const ctx = c.getContext("2d")!;
      ctx.fillStyle = "rgba(255,255,255,0.92)";
      ctx.fillRect(0, 0, 512, 128);
      ctx.fillStyle = "#111827";
      ctx.font = "bold 52px 'DM Sans', Arial, sans-serif";
      ctx.textAlign = "center";
      ctx.textBaseline = "middle";
      ctx.fillText(text, 256, 64);
      const tex = new THREE.CanvasTexture(c);
      tex.anisotropy = 4;
      return tex;
    }

    const handleResize = () => {
      const w = mount.clientWidth || window.innerWidth;
      const h = mount.clientHeight || window.innerHeight;
      camera.aspect = w / h;
      camera.updateProjectionMatrix();
      renderer.setSize(w, h);
    };
    window.addEventListener("resize", handleResize);

    const onClick = (e: MouseEvent) => {
      const rect = renderer.domElement.getBoundingClientRect();
      pointer.x = ((e.clientX - rect.left) / rect.width) * 2 - 1;
      pointer.y = -((e.clientY - rect.top) / rect.height) * 2 + 1;
      raycaster.setFromCamera(pointer, camera);
      const hits = raycaster.intersectObjects(cardMeshes, false);
      if (hits.length > 0) {
        const cb = hits[0].object.userData.onClick as (() => void) | undefined;
        cb?.();
      }
    };
    renderer.domElement.addEventListener("click", onClick);

    let raf = 0;
    const animate = () => {
      raf = requestAnimationFrame(animate);
      if (!reducedMotion) {
        scene.rotation.y += 0.0015;
      }
      controls.update();
      renderer.render(scene, camera);
    };
    animate();

    return () => {
      cancelAnimationFrame(raf);
      window.removeEventListener("resize", handleResize);
      renderer.domElement.removeEventListener("click", onClick);
      controls.dispose();
      renderer.dispose();
      geo_dispose(scene);
      if (renderer.domElement.parentNode === mount) {
        mount.removeChild(renderer.domElement);
      }
    };

    function geo_dispose(s: THREE.Scene) {
      s.traverse((obj) => {
        const m = obj as THREE.Mesh;
        if (m.geometry) m.geometry.dispose();
        if (m.material) {
          const mat = m.material as THREE.Material | THREE.Material[];
          if (Array.isArray(mat)) mat.forEach((x) => x.dispose());
          else mat.dispose();
        }
      });
    }
  }, [items, cameraPosition, reducedMotion, isMobile]);

  return (
    <div
      ref={mountRef}
      className="h-full w-full"
      style={{ width: "100%", height: "100%", minHeight: "60vh" }}
      role="img"
      aria-label="Interactive 3D product dashboard. Use the navigation menu to explore modules."
    />
  );
};
