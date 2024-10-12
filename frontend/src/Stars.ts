export class Stars {
  ctx: CanvasRenderingContext2D;
  canvas: HTMLCanvasElement;
  stars: { x: number; y: number; size: number }[] = [];
  frameCount: number = 0;
  frameDelay: number = 10;
  analyserNode: AnalyserNode | undefined;

  constructor(numStars: number) {
    const canvas = document.getElementById("canvas") as HTMLCanvasElement;

    this._resize(canvas);
    //window.addEventListener("resize", () => this._resize(this.canvas));

    this.canvas = canvas;
    this.ctx = canvas.getContext("2d") as CanvasRenderingContext2D;
    this.ctx.fillStyle = "white";
    this.ctx.strokeStyle = "white";

    for (let i = 0; i < numStars; i++) {
      this.stars.push({
        x: Math.random() * canvas.width,
        y: Math.random() * canvas.height,
        size: Math.random(),
      });
    }

    this.animate = this.animate.bind(this);

    this.animate();
  }

  setAnalyser(analyserNode: AnalyserNode) {
    this.analyserNode = analyserNode;
  }

  animate() {
    this.frameCount++;
    if (this.frameCount % this.frameDelay === 0) {
      this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);

      if (this.analyserNode) {
        const bufferLength = this.analyserNode.frequencyBinCount;
        const dataArray = new Uint8Array(bufferLength);
        this.analyserNode.getByteTimeDomainData(dataArray);

        const sliceWidth = this.canvas.width / bufferLength;
        let x = 0;

        for (let i = 0; i < bufferLength; i++) {
          const v = dataArray[i] / 128.0;
          const y = v * (this.canvas.height / 2);

          this.stars.forEach((star) => {
            const distance = Math.sqrt((star.x - x) ** 2 + (star.y - y) ** 2);
            const threshold = Math.random() * 50;
            let size = 1;

            if (distance < threshold) {
              size += Math.random() + 1;
            }

            this.ctx.beginPath();
            this.ctx.arc(
              star.x - star.size,
              star.y - star.size,
              star.size * size * 2,
              0,
              Math.PI * 2,
            );
            this.ctx.closePath();
            this.ctx.fill();
          });

          x += sliceWidth;
        }
      }
    }
    requestAnimationFrame(this.animate);
  }

  _resize(canvas: HTMLCanvasElement) {
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight * 0.75;
  }
}
