#!/usr/bin/env node
const WebSocket = require('ws');
const { execSync } = require('child_process');

const WS_URL = process.env.WS_URL || 'ws://localhost:9233';

let passed = 0;
let failed = 0;

function ok(label) { console.log(`  \u2713 PASS: ${label}`); passed++; }
function fail(label, expected, actual) {
  console.log(`  \u2717 FAIL: ${label}`);
  if (expected !== undefined) console.log(`    expected: ${JSON.stringify(expected)}`);
  if (actual !== undefined) console.log(`    actual:   ${JSON.stringify(actual)}`);
  failed++;
}

function sys(cmd) {
  try { return execSync(cmd, { encoding: 'utf-8', timeout: 10000 }).trim(); }
  catch { return ''; }
}

function invoke(ws, cmd, args = {}) {
  return new Promise((resolve, reject) => {
    const msg = JSON.stringify({ command: cmd, args });
    ws.send(msg);
    const handler = (data) => {
      try {
        const resp = JSON.parse(data.toString());
        if (resp.error) return reject(new Error(resp.error));
        resolve(resp.result);
      } catch (e) { reject(e); }
      finally { ws.removeListener('message', handler); }
    };
    ws.on('message', handler);
    setTimeout(() => { ws.removeListener('message', handler); reject(new Error('timeout')); }, 15000);
  });
}

async function testSystemInfo(stats) {
  console.log('\n[1] System Info (ler_hardware)');
  const si = stats.system_info;

  const sysTotalMem = parseInt(sys('sysctl -n hw.memsize'));
  ok(`total_memory matches sysctl (${sysTotalMem})`);
  if (stats.mem_info.total_memory !== sysTotalMem)
    fail('total_memory', sysTotalMem, stats.mem_info.total_memory);

  const sysCores = parseInt(sys('sysctl -n hw.ncpu'));
  ok(`cpu_cores matches sysctl (${sysCores})`);
  if (si.cpu_cores !== sysCores)
    fail('cpu_cores', sysCores, si.cpu_cores);

  const sysHostname = sys('hostname');
  ok(`hostname matches (${sysHostname})`);
  if (si.hostname !== sysHostname)
    fail('hostname', sysHostname, si.hostname);

  const sysKernel = sys('uname -r');
  ok(`kernel_version matches (${sysKernel})`);
  if (si.kernel_version !== sysKernel)
    fail('kernel_version', sysKernel, si.kernel_version);

  const sysOsName = sys('uname -s');
  ok(`os_name matches (${sysOsName})`);
  if (si.os_name !== sysOsName)
    fail('os_name', sysOsName, si.os_name);

  const sysOsVer = sys('sw_vers -productVersion');
  ok(`os_version matches (${sysOsVer})`);
  if (si.os_version !== sysOsVer)
    fail('os_version', sysOsVer, si.os_version);

  const boottime = parseInt(sys('sysctl -n kern.boottime | awk -F"[= ,]" \'{print $6}\''));
  const now = Math.floor(Date.now() / 1000);
  const sysUptime = now - boottime;
  const diff = Math.abs(si.uptime_secs - sysUptime);
  ok(`uptime within 10s (app: ${si.uptime_secs}s, sys: ${sysUptime}s, diff: ${diff}s)`);
  if (diff > 10)
    fail('uptime_secs too far', sysUptime, si.uptime_secs);

  const cpuBrand = sys('sysctl -n machdep.cpu.brand_string 2>/dev/null || echo "Apple M1"');
  ok(`cpu_brand matches (${cpuBrand})`);
  if (si.cpu_brand !== cpuBrand)
    fail('cpu_brand', cpuBrand, si.cpu_brand);

  ok(`total_processes is positive (${si.total_processes})`);
  if (si.total_processes <= 0)
    fail('total_processes should be > 0', '> 0', si.total_processes);

  ok('cpu_per_core has entries');
  if (!stats.cpu_per_core || stats.cpu_per_core.length === 0)
    fail('cpu_per_core should not be empty', 'non-empty', stats.cpu_per_core);

  ok(`cpu_usage in valid range [0,100] (${stats.cpu_usage.toFixed(1)}%)`);
  if (stats.cpu_usage < 0 || stats.cpu_usage > 100)
    fail('cpu_usage range', '[0,100]', stats.cpu_usage);
}

async function testMemory(mem) {
  console.log('\n[2] Memory (ler_hardware.mem_info)');

  ok('total_memory > 0');
  if (mem.total_memory <= 0) fail('total_memory should be > 0');

  const totalGb = mem.total_memory / 1073741824;
  ok(`total_memory seems reasonable (${totalGb.toFixed(1)} GB)`);
  if (totalGb < 0.5 || totalGb > 1024)
    fail('total_memory unrealistic', '0.5-1024 GB', `${totalGb} GB`);

  ok('used_memory <= total_memory');
  if (mem.used_memory > mem.total_memory)
    fail('used_memory exceeds total', `<= ${mem.total_memory}`, mem.used_memory);

  ok('free_memory <= total_memory');
  if (mem.free_memory > mem.total_memory)
    fail('free_memory exceeds total', `<= ${mem.total_memory}`, mem.free_memory);

  ok('available_memory <= total_memory');
  if (mem.available_memory > mem.total_memory)
    fail('available_memory exceeds total', `<= ${mem.total_memory}`, mem.available_memory);

  const calculatedUsed = mem.total_memory - mem.available_memory;
  const diff = Math.abs(mem.used_memory - calculatedUsed);
  ok(`used_memory \u2248 total - available (diff: ${(diff / 1048576).toFixed(1)} MB)`);
  if (diff > 1073741824)
    fail('used_memory should be close to total - available', `~${calculatedUsed}`, mem.used_memory);

  ok('swap values are 0 or positive');
  if (mem.used_swap < 0 || mem.total_swap < 0 || mem.free_swap < 0)
    fail('swap values should be >= 0');

  if (mem.breakdown) {
    ok('breakdown has all 4 fields');
    const b = mem.breakdown;
    if (b.app_memory === undefined || b.wired_memory === undefined || b.compressed_memory === undefined || b.cached_memory === undefined)
      fail('breakdown should have app, wired, compressed, cached');
  }
}

async function testDisk(disk) {
  console.log('\n[3] Disk (ler_hardware.disk_usage)');

  ok('total_bytes > 0');
  if (disk.total_bytes <= 0) fail('total_bytes should be > 0');

  const totalGb = disk.total_bytes / 1073741824;
  ok(`total_bytes seems reasonable (${totalGb.toFixed(0)} GB)`);
  if (totalGb < 1 || totalGb > 100000)
    fail('total_bytes unrealistic', '1-100000 GB', `${totalGb} GB`);

  ok('available_bytes <= total_bytes');
  if (disk.available_bytes > disk.total_bytes)
    fail('available_bytes exceeds total', `<= ${disk.total_bytes}`, disk.available_bytes);
}

async function testProcesses(procs) {
  console.log('\n[4] Processes (ler_hardware.processes)');

  ok('processes list is non-empty');
  if (!procs || procs.length === 0) fail('processes should not be empty');

  if (procs && procs.length > 0) {
    ok(`at most 15 processes (got ${procs.length})`);
    if (procs.length > 15) fail('processes should be <= 15', '<= 15', procs.length);

    ok('processes sorted by memory_usage descending');
    const sorted = [...procs].sort((a, b) => b.memory_usage - a.memory_usage);
    const isSorted = procs.every((p, i) => p.memory_usage === sorted[i].memory_usage);
    if (!isSorted) fail('processes not sorted by memory');

    const validPids = procs.filter(p => p.pid > 0);
    ok(`all ${validPids.length}/${procs.length} processes have valid PIDs`);
    if (validPids.length !== procs.length)
      fail('some pids are 0');

    const validMem = procs.filter(p => p.memory_usage > 0);
    ok(`all ${validMem.length}/${procs.length} processes have memory > 0`);
    if (validMem.length !== procs.length)
      fail('some memory_usage values are 0');
  }
}

async function testNetwork(net) {
  console.log('\n[5] Network Stats (ler_rede.stats)');

  ok('stats.total > 0');
  if (!net.stats || net.stats.total <= 0) fail('total connections should be > 0');

  const s = net.stats;
  ok('TCP + UDP = total');
  if (s.tcp + s.udp !== s.total)
    fail('tcp + udp should equal total', s.total, s.tcp + s.udp);

  ok('established connections \u2264 total');
  if (s.established > s.total)
    fail('established exceeds total');

  ok('listening connections \u2264 total');
  if (s.listening > s.total)
    fail('listening exceeds total');

  ok('traffic.physical has en0 (WiFi)');
  const hasEn0 = net.traffic.physical.some(i => i.name === 'en0');
  if (!hasEn0) fail('en0 should be in physical interfaces');

  const en0 = net.traffic.physical.find(i => i.name === 'en0');
  if (en0) {
    ok('en0 has IP addresses');
    if (!en0.ip_addresses || en0.ip_addresses.length === 0)
      fail('en0 should have IPs');

    const hasIPv4 = en0.ip_addresses.some(ip => ip.includes('.'));
    ok(`en0 has IPv4 (${hasIPv4})`);
  }

  ok('lo0 (loopback) in special interfaces');
  const hasLo0 = net.traffic.special.some(i => i.name === 'lo0');
  if (!hasLo0) fail('lo0 should be in special interfaces');
}

async function testLocalNetwork(localNet) {
  console.log('\n[6] Local Network (get_local_network_info)');

  ok('local_ip is present');
  if (!localNet.local_ip) fail('local_ip should not be empty');
  if (localNet.local_ip === '127.0.0.1')
    fail('local_ip should not be loopback');

  const sysLocalIp = sys("ifconfig en0 2>/dev/null | grep 'inet ' | awk '{print $2}'");
  if (sysLocalIp) {
    ok(`local_ip matches ifconfig (${localNet.local_ip})`);
    if (localNet.local_ip !== sysLocalIp)
      fail('local_ip differs from ifconfig', sysLocalIp, localNet.local_ip);
  }

  ok('gateway is present');
  if (!localNet.gateway) fail('gateway should not be empty');

  const sysGateway = sys("netstat -rn -f inet 2>/dev/null | grep '^default' | awk '{print $2}'");
  if (sysGateway) {
    ok(`gateway matches netstat (${localNet.gateway})`);
    if (localNet.gateway !== sysGateway)
      fail('gateway differs from netstat', sysGateway, localNet.gateway);
  }

  ok('subnet_mask is present');
  if (!localNet.subnet_mask) fail('subnet_mask should not be empty');

  ok('dns_servers has entries');
  if (!localNet.dns_servers || localNet.dns_servers.length === 0)
    fail('dns_servers should not be empty');

  ok('devices list has entries');
  if (!localNet.devices || localNet.devices.length === 0)
    fail('devices should show at least gateway');

  const matchGateway = localNet.devices.some(d => d.ip === localNet.gateway);
  ok(`gateway ${localNet.gateway} appears in devices (${matchGateway})`);
}

async function testInternetInfo(inet) {
  console.log('\n[7] Internet Info (get_internet_info)');

  ok('online status is boolean');
  if (typeof inet.info.online !== 'boolean')
    fail('online should be boolean');

  if (inet.info.online) {
    ok('public_ip is non-empty when online');
    if (!inet.info.public_ip) fail('public_ip should not be empty when online');

    ok('isp is non-empty');
    if (!inet.info.isp) fail('isp should not be empty when online');

    ok('latency_ms is positive');
    if (inet.info.latency_ms <= 0)
      fail('latency should be > 0 when online');
  }

  ok('ping_target is cloudflare');
  if (inet.info.ping_target !== 'Cloudflare (1.1.1.1)')
    fail('ping_target', 'Cloudflare (1.1.1.1)', inet.info.ping_target);

  // SSID accuracy test
  if (inet.info.online) {
    const sysSsid = sys("/usr/sbin/ipconfig getsummary en0 2>/dev/null | grep 'SSID : ' | sed 's/^[[:space:]]*SSID : //'");
    if (sysSsid) {
      ok(`wifi_ssid matches system command (${inet.info.wifi_ssid})`);
      if (inet.info.wifi_ssid !== sysSsid)
        fail('wifi_ssid differs from ipconfig', sysSsid, inet.info.wifi_ssid);
    } else {
      // Could be on ethernet, no WiFi — then null is acceptable
      ok('wifi_ssid is null (no WiFi interface or ethernet)');
      if (inet.info.wifi_ssid !== null)
        fail('wifi_ssid should be null when ipconfig returns no SSID');
    }
  }
}

async function testHistory(hist) {
  console.log('\n[8] History (obter_historico)');

  if (hist.sistema && hist.sistema.length > 0) {
    ok('history sistema has entries');
    const first = hist.sistema[0];
    ok('cpu_global is valid in range');
    if (first.cpu_global < 0 || first.cpu_global > 100)
      fail('cpu_global range', '[0,100]', first.cpu_global);
    ok('ram_usada > 0');
    if (first.ram_usada <= 0) fail('ram_usada should be > 0');
  }

  if (hist.processos && hist.processos.length > 0) {
    ok('history processos has entries');
    const first = hist.processos[0];
    ok('processo cpu is valid');
    if (first.cpu < 0) fail('cpu should be >= 0');
    ok('processo memoria > 0');
    if (first.memoria <= 0) fail('memoria should be > 0');
    ok('processo nome is non-empty');
    if (!first.nome) fail('process name should not be empty');
  }
}

async function testAggregatedProcesses(procs) {
  console.log('\n[9] Aggregated Processes (obter_processos_agrupados)');

  if (procs && procs.length > 0) {
    ok('aggregated processes has entries');
    const first = procs[0];
    ok('total_cpu is valid');
    if (first.total_cpu < 0) fail('total_cpu should be >= 0');
    ok('total_memoria > 0');
    if (first.total_memoria <= 0) fail('total_memoria should be > 0');
    ok('nome is non-empty');
    if (!first.nome) fail('nome should not be empty');
  }
}

async function main() {
  const ws = new WebSocket(WS_URL);

  ws.on('open', async () => {
    console.log(`Connected to ${WS_URL}`);
    console.log('============================================');
    console.log(' InSysIn \u2014 Accuracy Verification Suite');
    console.log('============================================');

    try {
      const stats = await invoke(ws, 'ler_hardware');
      await testSystemInfo(stats);
      await testMemory(stats.mem_info);
      await testDisk(stats.disk_usage);
      await testProcesses(stats.processes);

      const net = await invoke(ws, 'ler_rede');
      await testNetwork(net);

      const localNet = await invoke(ws, 'get_local_network_info');
      await testLocalNetwork(localNet);

      const inet = await invoke(ws, 'get_internet_info');
      await testInternetInfo(inet);

      const hist = await invoke(ws, 'obter_historico');
      await testHistory(hist);

      const aggProcs = await invoke(ws, 'obter_processos_agrupados', { ordem: 'cpu', desc: true });
      await testAggregatedProcesses(aggProcs);

    } catch (err) {
      console.error(`\n  \u2717 ERROR: ${err.message}`);
      failed++;
    } finally {
      ws.close();
    }

    console.log('\n============================================');
    console.log(` Results: ${passed} passed, ${failed} failed`);
    console.log('============================================');
    process.exit(failed > 0 ? 1 : 0);
  });

  ws.on('error', (err) => {
    console.error(`Connection error: ${err.message}`);
    console.error(`Make sure the Tauri app is running (WS bridge on ${WS_URL})`);
    process.exit(1);
  });
}

main();
