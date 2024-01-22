from scapy.all import *
import threading
import socket

request_dict = {}

def handle_icmp_request(packet):
    if packet.haslayer(ICMP) and packet[ICMP].type == 8:
        original_sender_ip = packet[IP].src
        icmp_id = packet[ICMP].id
        icmp_seq = packet[ICMP].seq
        print(f"Received ICMP request from {original_sender_ip} with id={icmp_id}, seq={icmp_seq}")

        request_dict[(icmp_id, icmp_seq)] = original_sender_ip

        raw_data = packet[Raw].load
        ip_address = raw_data[16:20]
        ip_str = socket.inet_ntoa(ip_address)
        print(f"Extracted IP address from payload: {ip_str}")

        ip = IP(dst=ip_str)
        icmp = ICMP(id=icmp_id, seq=icmp_seq)
        raw = Raw(load=raw_data)
        send(ip/icmp/raw)
        print(f"Forwarded ICMP request to {ip_str}")

def handle_icmp_reply(packet):
    if packet.haslayer(ICMP) and packet[ICMP].type == 0:
        icmp_id = packet[ICMP].id
        icmp_seq = packet[ICMP].seq
        print(f"Received ICMP reply with id={icmp_id}, seq={icmp_seq}")

        if (icmp_id, icmp_seq) in request_dict:
            original_sender_ip = request_dict[(icmp_id, icmp_seq)]
            print(f"Original sender IP for id={icmp_id}, seq={icmp_seq} is {original_sender_ip}")

            ip = IP(dst=original_sender_ip)
            icmp = ICMP(type=0, id=icmp_id, seq=icmp_seq)
            raw = packet[Raw].load
            send(ip/icmp/raw)
            print(f"Forwarded ICMP reply to {original_sender_ip}")

thread1 = threading.Thread(target=sniff, kwargs={'iface': 'wlp47s0', 'filter': 'icmp', 'prn': handle_icmp_request})
thread2 = threading.Thread(target=sniff, kwargs={'iface': 'anp0', 'filter': 'icmp', 'prn': handle_icmp_reply})

thread1.start()
thread2.start()

thread1.join()
thread2.join()
