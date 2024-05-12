import Mathlib.Data.Set.Basic
import Mathlib.Topology.MetricSpace.Basic

class Voronoi (γ : Type w) [MetricSpace γ] (n : Nat) (α : Type u) where
  sites : α → Fin n → γ
  cell : α → Fin n → Set γ
  cell_dist : (a : α) → ∀ k : Fin n, ∀ p ∈ cell a k, ∀ j ≠ k, dist p (sites a k) ≤ dist p (sites a j)

def SimpleVoronoi (γ : Type w) [MetricSpace γ] (n : Nat) : Type w := Vector γ n

@[simp]
def SimpleVoronoi.sites {γ : Type w} [MetricSpace γ] {n : Nat} (sv : SimpleVoronoi γ n) : Fin n → γ :=
  λ ι ↦ sv.get ι

@[simp]
def SimpleVoronoi.cell {γ : Type w} [MetricSpace γ] {n : Nat} (sv : SimpleVoronoi γ n) : Fin n → Set γ :=
  λ k ↦
    λ p ↦ (List.finRange n).filter λ j ↦ j ≠ k ∧ dist p (sv.sites j) < dist p (sv.sites k) = []

def SimpleVoronoi.cell_dist {γ : Type w} [MetricSpace γ] (n : Nat) (sv : SimpleVoronoi γ n)
  : ∀ k : Fin n, ∀ p ∈ sv.cell k, ∀ j : Fin n, j ≠ k → dist p (sv.sites k) ≤ dist p (sv.sites j) := by
  intro k p hp j h_ne
  simp
  simp [Membership.mem, Set.Mem] at hp
  have := List.filter_eq_nil.mp hp j (List.mem_finRange j)
  simp at this
  have := this h_ne
  exact this

instance (γ : Type w) [inst : MetricSpace γ] (n : Nat) : Voronoi γ n (SimpleVoronoi γ n) where
  sites := SimpleVoronoi.sites
  cell := SimpleVoronoi.cell
  cell_dist := @SimpleVoronoi.cell_dist γ inst n
