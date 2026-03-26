#!/usr/bin/env python3
"""
CLI pour gérer le superviseur Nook
"""
import argparse
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent))

from supervisor.supervisor import NookSupervisor

def main():
    parser = argparse.ArgumentParser(description="Superviseur Nook")
    subparsers = parser.add_subparsers(dest="command")
    
    # Commande: status
    status_parser = subparsers.add_parser("status", help="État du superviseur")
    
    # Commande: optimize
    optimize_parser = subparsers.add_parser("optimize", help="Rapport d'optimisation")
    
    # Commande: migrate
    migrate_parser = subparsers.add_parser("migrate", help="Migrer les rôles vers agents")
    
    args = parser.parse_args()
    supervisor = NookSupervisor()
    
    if args.command == "status":
        report = supervisor.get_optimization_report()
        print(json.dumps(report, indent=2))
    
    elif args.command == "optimize":
        report = supervisor.get_optimization_report()
        print("📊 Rapport d'optimisation:")
        print(f"  Coût total: {report.get('total_cost')}")
        print(f"  Tâches traitées: {report.get('tasks_processed')}")
        for opt in report.get('optimizations', []):
            print(f"  ⚡ {opt['type']}: {opt.get('potential_savings')}")
    
    elif args.command == "migrate":
        from supervisor.migrate_roles import migrate_roles
        migrate_roles()
    
    else:
        parser.print_help()

if __name__ == "__main__":
    main()