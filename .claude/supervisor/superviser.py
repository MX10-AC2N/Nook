"""
Superviseur principal pour Nook
Orchestre les agents et optimise les coûts
"""
import json
import yaml
import asyncio
from pathlib import Path
from datetime import datetime
from typing import Dict, Any, Optional, List
from .classifier import TaskClassifier, TaskComplexity

class NookSupervisor:
    def __init__(self, config_path: str = ".claude/supervisor/config.yaml"):
        self.config = self._load_config(config_path)
        self.classifier = TaskClassifier()
        self.metrics: List[Dict[str, Any]] = []
        self.daily_spending = 0.0
    
    def _load_config(self, path: str) -> Dict:
        """Charge la configuration du superviseur"""
        config_file = Path(path)
        if config_file.exists():
            with open(config_file, 'r') as f:
                return yaml.safe_load(f)
        return {}
    
    async def process_task(self, task: str, role: Optional[str] = None) -> Dict[str, Any]:
        """
        Traite une tâche en sélectionnant l'agent et le modèle optimaux
        """
        start_time = datetime.now()
        
        # 1. Classifier la tâche
        complexity = self.classifier.classify(task)
        recommended_model = self.classifier.get_model_recommendation(complexity)
        
        # 2. Vérifier les limites de budget
        if not self._check_budget_limit():
            return {
                "error": "Budget quotidien dépassé",
                "suggestion": "Réessaie demain ou augmente le budget"
            }
        
        # 3. Sélectionner l'agent approprié
        agent_id = self._select_agent(role, complexity)
        
        # 4. Exécuter la tâche
        result = await self._execute_task(agent_id, task, recommended_model)
        
        # 5. Enregistrer les métriques
        metrics = self._record_metrics(
            task=task,
            agent_id=agent_id,
            model=recommended_model,
            complexity=complexity.value,
            result=result,
            start_time=start_time
        )
        
        return {
            "success": True,
            "result": result.get("output"),
            "metadata": {
                "agent_id": agent_id,
                "model_used": recommended_model,
                "complexity": complexity.value,
                "cost_estimate": metrics["cost"],
                "tokens_used": metrics["tokens"]
            }
        }
    
    def _select_agent(self, role: Optional[str], complexity: TaskComplexity) -> str:
        """Sélectionne l'agent le plus approprié"""
        if role:
            # Utiliser le rôle spécifié
            return role
        
        # Sinon, sélectionner automatiquement basé sur la tâche
        roles_dir = Path(self.config.get("supervisor", {}).get(
            "roles_integration", {}
        ).get("roles_path", ".claude/roles"))
        
        if roles_dir.exists():
            role_files = list(roles_dir.glob("*.md"))
            if role_files:
                # Retourner le premier rôle disponible
                return role_files[0].stem
        
        return "default"
    
    async def _execute_task(self, agent_id: str, task: str, model: str) -> Dict[str, Any]:
        """Exécute la tâche via l'agent sélectionné"""
        # TODO: Intégrer avec l'API Claude réelle
        # Ceci est un placeholder pour l'implémentation
        return {
            "output": f"[Résultat simulé pour {agent_id} avec {model}]",
            "tokens_used": 100,
            "cost": 0.01
        }
    
    def _record_metrics(self, **kwargs) -> Dict[str, Any]:
        """Enregistre les métriques d'exécution"""
        metrics = {
            "timestamp": datetime.now().isoformat(),
            "cost": kwargs.get("result", {}).get("cost", 0),
            "tokens": kwargs.get("result", {}).get("tokens_used", 0),
            **kwargs
        }
        self.metrics.append(metrics)
        self.daily_spending += metrics["cost"]
        return metrics
    
    def _check_budget_limit(self) -> bool:
        """Vérifie si le budget quotidien n'est pas dépassé"""
        limits = self.config.get("supervisor", {}).get("cost_limits", {})
        daily_budget = limits.get("daily_budget", 50.0)
        
        return self.daily_spending < daily_budget
    
    def get_optimization_report(self) -> Dict[str, Any]:
        """Génère un rapport d'optimisation des coûts"""
        if not self.metrics:
            return {"message": "Aucune donnée disponible"}
        
        total_cost = sum(m["cost"] for m in self.metrics)
        total_tokens = sum(m["tokens"] for m in self.metrics)
        
        # Analyser les opportunités d'optimisation
        optimizations = []
        for metric in self.metrics:
            if metric["complexity"] == "simple" and metric["model"] == "opus":
                optimizations.append({
                    "type": "downgrade_possible",
                    "agent": metric["agent_id"],
                    "current_model": metric["model"],
                    "recommended_model": "haiku",
                    "potential_savings": "~80%"
                })
        
        return {
            "total_cost": f"${total_cost:.4f}",
            "total_tokens": total_tokens,
            "tasks_processed": len(self.metrics),
            "avg_cost_per_task": f"${total_cost / len(self.metrics):.4f}",
            "optimizations": optimizations[:10],
            "daily_spending": f"${self.daily_spending:.4f}"
        }