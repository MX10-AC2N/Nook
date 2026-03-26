"""
Classification intelligente des tâches pour Nook
Adapté aux rôles existants dans .claude/roles
"""
import re
from enum import Enum
from pathlib import Path
from typing import Dict, List, Optional

class TaskComplexity(Enum):
    SIMPLE = "simple"
    MODERATE = "moderate"
    COMPLEX = "complex"
    CRITICAL = "critical"

class TaskClassifier:
    def __init__(self, roles_dir: str = ".claude/roles"):
        self.roles_dir = Path(roles_dir)
        self.role_keywords = self._load_role_keywords()
    
    def _load_role_keywords(self) -> Dict[str, List[str]]:
        """Charge les mots-clés spécifiques aux rôles de Nook"""
        keywords = {
            TaskComplexity.SIMPLE: [
                "bonjour", "merci", "aide", "comment", "qu'est-ce que",
                "info", "date", "heure", "version"
            ],
            TaskComplexity.MODERATE: [
                "analyser", "résumer", "expliquer", "générer", "créer",
                "modifier", "améliorer", "documenter", "tester"
            ],
            TaskComplexity.COMPLEX: [
                "architecture", "design pattern", "optimisation",
                "refactoring", "debug", "performance", "intégration"
            ],
            TaskComplexity.CRITICAL: [
                "sécurité", "audit", "production", "données sensibles",
                "paiement", "authentification", "crypto"
            ]
        }
        
        # Ajouter les mots-clés spécifiques aux rôles existants
        if self.roles_dir.exists():
            for role_file in self.roles_dir.glob("*.md"):
                role_name = role_file.stem
                content = role_file.read_text().lower()
                
                # Extraire les compétences du rôle
                if "expert" in content or "senior" in content:
                    keywords[TaskComplexity.COMPLEX].extend([
                        f"{role_name}", f"{role_name} expert"
                    ])
        
        return keywords
    
    def classify(self, task: str, context: Optional[Dict] = None) -> TaskComplexity:
        """Classifie une tâche selon sa complexité"""
        text = task.lower()
        
        # Score basé sur les mots-clés
        scores = {complexity: 0 for complexity in TaskComplexity}
        
        for complexity, keywords in self.role_keywords.items():
            for keyword in keywords:
                if keyword in text:
                    scores[complexity] += 1
        
        # Déterminer la complexité dominante
        max_score = max(scores.values())
        if max_score == 0:
            # Fallback basé sur la longueur et la structure
            word_count = len(text.split())
            if word_count < 15:
                return TaskComplexity.SIMPLE
            elif word_count < 50:
                return TaskComplexity.MODERATE
            else:
                return TaskComplexity.COMPLEX
        
        # Retourner la complexité avec le score le plus élevé
        for complexity, score in scores.items():
            if score == max_score:
                return complexity
        
        return TaskComplexity.MODERATE
    
    def get_model_recommendation(self, complexity: TaskComplexity) -> str:
        """Retourne le modèle recommandé pour une complexité donnée"""
        mapping = {
            TaskComplexity.SIMPLE: "haiku",
            TaskComplexity.MODERATE: "sonnet",
            TaskComplexity.COMPLEX: "opus",
            TaskComplexity.CRITICAL: "opus"
        }
        return mapping.get(complexity, "sonnet")