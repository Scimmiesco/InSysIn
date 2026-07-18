import { ComponentFixture, TestBed } from "@angular/core/testing";
import { signal, WritableSignal } from "@angular/core";
import { Internet } from "./internet.component";
import { InternetStore } from "../../store/internet.store";
import type { InternetDiagnostics, LocalNetworkInfo } from "../../../generated/types";

interface MockStore extends InternetStore {
  _diagnostics: WritableSignal<InternetDiagnostics | null>;
  _scanning: WritableSignal<boolean>;
  _error: WritableSignal<string | null>;
  _runningTest: WritableSignal<boolean>;
  _speedError: WritableSignal<string | null>;
  _scanningLocal: WritableSignal<boolean>;
  _localNetwork: WritableSignal<LocalNetworkInfo | null>;
}

function createMockStore(): MockStore {
  const _diagnostics = signal<InternetDiagnostics | null>(null);
  const _scanning = signal(true);
  const _error = signal<string | null>(null);
  const _runningTest = signal(false);
  const _speedError = signal<string | null>(null);
  const _scanningLocal = signal(true);
  const _localNetwork = signal<LocalNetworkInfo | null>(null);

  return {
    _diagnostics, _scanning, _error, _runningTest, _speedError, _scanningLocal, _localNetwork,
    diagnostics: _diagnostics.asReadonly(),
    scanning: _scanning.asReadonly(),
    error: _error.asReadonly(),
    runningTest: _runningTest.asReadonly(),
    speedError: _speedError.asReadonly(),
    scanningLocal: _scanningLocal.asReadonly(),
    localNetwork: _localNetwork.asReadonly(),
    load: () => {},
    refreshAll: () => {},
    runSpeedTest: () => Promise.resolve(),
    scanLocal: () => {},
  } as MockStore;
}

function makeDiagnostics(ssid: string | null): InternetDiagnostics {
  return {
    info: {
      public_ip: "1.2.3.4",
      isp: "Test ISP",
      city: "Test City",
      country: "Test Country",
      org: "Test Org",
      timezone: "UTC",
      asn: "AS12345",
      latency_ms: 10.5,
      ping_target: "Cloudflare (1.1.1.1)",
      online: true,
      wifi_ssid: ssid,
    },
    speed: null,
  };
}

describe("Internet Component — WiFi SSID Rendering", () => {
  let fixture: ComponentFixture<Internet>;
  let store: MockStore;

  function setup() {
    TestBed.configureTestingModule({
      imports: [Internet],
      providers: [{ provide: InternetStore, useValue: store }],
    });
    fixture = TestBed.createComponent(Internet);
  }

  function renderedText(): string {
    return (fixture.nativeElement as HTMLElement).textContent ?? "";
  }

  it("[1] renders wifi_ssid value when provided", () => {
    store = createMockStore();
    store._diagnostics.set(makeDiagnostics("Sem_Rede-5G"));
    store._scanning.set(false);

    setup();
    fixture.detectChanges();

    const text = renderedText();
    console.log("=== [1] SSID PROVIDED ===");
    console.log("Rendered text:", text);
    console.log("========================");

    expect(text).toContain("Sem_Rede-5G");
    expect(text).toContain("WiFi");
  });

  it("[2] renders em dash when wifi_ssid is null", () => {
    store = createMockStore();
    store._diagnostics.set(makeDiagnostics(null));
    store._scanning.set(false);

    setup();
    fixture.detectChanges();

    const text = renderedText();
    console.log("=== [2] SSID NULL ===");
    console.log("Rendered text:", text);
    console.log("=====================");

    expect(text).toContain("WiFi");
    expect(text).toContain("—");
  });

  it("[3] shows skeleton when diagnostics is null (loading)", () => {
    store = createMockStore();
    store._scanning.set(true);

    setup();
    fixture.detectChanges();

    const text = renderedText();
    console.log("=== [3] LOADING STATE ===");
    console.log("Rendered text:", text);
    console.log("=========================");

    const infoItems = fixture.nativeElement.querySelectorAll("ui-info-item");
    expect(infoItems.length).toBe(0);
    expect(text).toContain("Checking");
  });

  it("[4] renders all connection fields including WiFi", () => {
    store = createMockStore();
    store._diagnostics.set(makeDiagnostics("MyNetwork"));
    store._scanning.set(false);

    setup();
    fixture.detectChanges();

    const text = renderedText();
    console.log("=== [4] ALL FIELDS ===");
    console.log("Rendered text:", text);
    console.log("=====================");

    expect(text).toContain("Your IP Address");
    expect(text).toContain("1.2.3.4");
    expect(text).toContain("ISP");
    expect(text).toContain("Test ISP");
    expect(text).toContain("WiFi");
    expect(text).toContain("MyNetwork");
    expect(text).toContain("Online");
    expect(text).toContain("Ping");
    expect(text).toContain("10.5");
  });
});
