interface PerformanceMetrics {
  averageProcessingTime: number;
  messageBacklog: number;
  droppedMessages: number;
  processingTimes: number[];
}

export class WebSocketPerformanceMonitor {
  private metrics: PerformanceMetrics = {
    averageProcessingTime: 0,
    messageBacklog: 0,
    droppedMessages: 0,
    processingTimes: []
  };

  private readonly maxSamples = 100;

  recordProcessingTime(time: number) {
    this.metrics.processingTimes.push(time);
    if (this.metrics.processingTimes.length > this.maxSamples) {
      this.metrics.processingTimes.shift();
    }
    
    this.metrics.averageProcessingTime = 
      this.metrics.processingTimes.reduce((a, b) => a + b, 0) / 
      this.metrics.processingTimes.length;
  }

  updateBacklog(count: number) {
    this.metrics.messageBacklog = count;
  }

  recordDroppedMessage() {
    this.metrics.droppedMessages++;
  }

  getMetrics(): PerformanceMetrics {
    return { ...this.metrics };
  }

  shouldAdjustCapabilities(): boolean {
    return this.metrics.averageProcessingTime > 16.67; // 60fps threshold
  }
} 