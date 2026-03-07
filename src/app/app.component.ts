import { Component, OnInit, OnDestroy } from "@angular/core";
import { CommonModule } from "@angular/common";
import { invoke } from "@tauri-apps/api/core";

interface SystemStats {
  cpu_usage: number;
  memory_used: number;
  memory_total: number;
  os_name: string;
  os_version: string;
}

@Component({
  selector: "app-root",
  standalone: true,
  imports: [CommonModule],
  templateUrl: "./app.component.html",
  styleUrl: "./app.component.css",
})
export class AppComponent implements OnInit, OnDestroy {
  systemStats: SystemStats | null = null;
  interval: any;

  ngOnInit() {
    this.fetchStats();
    this.interval = setInterval(() => {
      this.fetchStats();
    }, 1000);
  }

  ngOnDestroy() {
    if (this.interval) {
      clearInterval(this.interval);
    }
  }

  async fetchStats() {
    try {
      this.systemStats = await invoke<SystemStats>("get_system_stats");
    } catch (e) {
      console.error("Failed to fetch system stats", e);
    }
  }

  getMemoryPercentage(): number {
    if (!this.systemStats) return 0;
    return (this.systemStats.memory_used / this.systemStats.memory_total) * 100;
  }

  formatBytes(bytes: number): string {
    if (bytes === 0) return "0 Bytes";
    const k = 1024;
    const sizes = ["Bytes", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
  }
}
